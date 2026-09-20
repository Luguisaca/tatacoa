use crate::{Error, Result};

pub(crate) const HEADER_BYTES: usize = 136;
pub(crate) const HEADER_AAD_BYTES: usize = 88;
pub(crate) const CHUNK_BYTES: u32 = 1_048_576;
pub(crate) const MAX_ENVELOPE_BYTES: u64 = 1_099_511_627_776;
pub(crate) const MAX_DIRECTORY_CIPHERTEXT_BYTES: u64 = 16 * 1024 * 1024;
pub(crate) const MAX_OBJECTS: u32 = 65_536;
pub(crate) const MAX_OBJECT_ID_BYTES: usize = 64;
pub(crate) const MAX_MANIFEST_BYTES: u64 = 16 * 1024 * 1024;
pub(crate) const MAX_ARTIFACT_BYTES: u64 = 256 * 1024 * 1024 * 1024;
pub(crate) const MAX_CHUNKS: u64 = 262_144;
pub(crate) const GCM_TAG_BYTES: u64 = 16;

const MAGIC: [u8; 16] = *b"TATACOAENCV1\0\0\0\0";
const DIRECTORY_MAGIC: [u8; 8] = *b"TATDIR01";

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct EncryptedHeader {
    pub(crate) salt: [u8; 16],
    pub(crate) wrap_nonce: [u8; 12],
    pub(crate) directory_nonce: [u8; 7],
    pub(crate) directory_ciphertext_length: u64,
    pub(crate) wrapped_bundle_key: [u8; 48],
}

impl EncryptedHeader {
    pub(crate) fn encode(&self) -> Result<[u8; HEADER_BYTES]> {
        validate_directory_ciphertext_length(self.directory_ciphertext_length)?;
        let mut bytes = [0_u8; HEADER_BYTES];
        bytes[0..16].copy_from_slice(&MAGIC);
        bytes[16..18].copy_from_slice(&(HEADER_BYTES as u16).to_be_bytes());
        bytes[20] = 1;
        bytes[21] = 1;
        bytes[22] = 1;
        bytes[23] = 1;
        bytes[24..28].copy_from_slice(&65_536_u32.to_be_bytes());
        bytes[28..32].copy_from_slice(&3_u32.to_be_bytes());
        bytes[32..36].copy_from_slice(&4_u32.to_be_bytes());
        bytes[36..40].copy_from_slice(&CHUNK_BYTES.to_be_bytes());
        bytes[40..56].copy_from_slice(&self.salt);
        bytes[56..68].copy_from_slice(&self.wrap_nonce);
        bytes[68..75].copy_from_slice(&self.directory_nonce);
        bytes[80..88].copy_from_slice(&self.directory_ciphertext_length.to_be_bytes());
        bytes[88..136].copy_from_slice(&self.wrapped_bundle_key);
        Ok(bytes)
    }

    pub(crate) fn decode(bytes: &[u8], envelope_length: u64) -> Result<Self> {
        if bytes.len() != HEADER_BYTES || envelope_length > MAX_ENVELOPE_BYTES {
            return invalid_envelope();
        }
        if bytes[0..16] != MAGIC
            || read_u16(bytes, 16)? != HEADER_BYTES as u16
            || read_u16(bytes, 18)? != 0
            || bytes[20..24] != [1, 1, 1, 1]
            || read_u32(bytes, 24)? != 65_536
            || read_u32(bytes, 28)? != 3
            || read_u32(bytes, 32)? != 4
            || read_u32(bytes, 36)? != CHUNK_BYTES
            || bytes[75..80].iter().any(|byte| *byte != 0)
        {
            return invalid_envelope();
        }
        let directory_ciphertext_length = read_u64(bytes, 80)?;
        validate_directory_ciphertext_length(directory_ciphertext_length)?;
        let minimum = u64::try_from(HEADER_BYTES)
            .map_err(|_| Error::Cryptography("invalid encrypted envelope"))?
            .checked_add(directory_ciphertext_length)
            .ok_or(Error::Cryptography("invalid encrypted envelope"))?;
        if minimum > envelope_length {
            return invalid_envelope();
        }
        Ok(Self {
            salt: copy_array(bytes, 40)?,
            wrap_nonce: copy_array(bytes, 56)?,
            directory_nonce: copy_array(bytes, 68)?,
            directory_ciphertext_length,
            wrapped_bundle_key: copy_array(bytes, 88)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ObjectType {
    Manifest,
    Artifact,
}

impl ObjectType {
    fn code(self) -> u8 {
        match self {
            Self::Manifest => 1,
            Self::Artifact => 2,
        }
    }

    fn from_code(code: u8) -> Result<Self> {
        match code {
            1 => Ok(Self::Manifest),
            2 => Ok(Self::Artifact),
            _ => invalid_envelope(),
        }
    }

    fn plaintext_limit(self) -> u64 {
        match self {
            Self::Manifest => MAX_MANIFEST_BYTES,
            Self::Artifact => MAX_ARTIFACT_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObjectDescriptor {
    pub(crate) object_type: ObjectType,
    pub(crate) object_id: String,
    pub(crate) plaintext_length: u64,
    pub(crate) ciphertext_length: u64,
    pub(crate) stream_nonce: [u8; 7],
}

impl ObjectDescriptor {
    fn encode(&self, output: &mut Vec<u8>) -> Result<()> {
        validate_descriptor(self)?;
        output.push(self.object_type.code());
        output.push(0);
        let id_length = u16::try_from(self.object_id.len())
            .map_err(|_| Error::Cryptography("invalid encrypted envelope"))?;
        output.extend_from_slice(&id_length.to_be_bytes());
        output.extend_from_slice(&self.plaintext_length.to_be_bytes());
        output.extend_from_slice(&self.ciphertext_length.to_be_bytes());
        output.extend_from_slice(&self.stream_nonce);
        output.push(0);
        output.extend_from_slice(self.object_id.as_bytes());
        Ok(())
    }

    pub(crate) fn canonical_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(28 + self.object_id.len());
        self.encode(&mut bytes)?;
        Ok(bytes)
    }
}

pub(crate) fn directory_aad(header: &[u8; HEADER_BYTES]) -> Vec<u8> {
    let mut aad = Vec::with_capacity(HEADER_BYTES + 37);
    aad.extend_from_slice(header);
    aad.extend_from_slice(b"TATACOA\0encrypted\0v1\0directory\0root");
    aad
}

pub(crate) fn object_aad(
    header: &[u8; HEADER_BYTES],
    descriptor: &ObjectDescriptor,
) -> Result<Vec<u8>> {
    let encoded = descriptor.canonical_bytes()?;
    let mut aad = Vec::with_capacity(HEADER_BYTES + 30 + encoded.len());
    aad.extend_from_slice(header);
    aad.extend_from_slice(b"TATACOA\0encrypted\0v1\0object\0");
    aad.extend_from_slice(&encoded);
    Ok(aad)
}

pub(crate) fn encode_directory(descriptors: &[ObjectDescriptor]) -> Result<Vec<u8>> {
    validate_descriptor_set(descriptors)?;
    let mut output = Vec::new();
    output.extend_from_slice(&DIRECTORY_MAGIC);
    output.extend_from_slice(
        &u32::try_from(descriptors.len())
            .map_err(|_| Error::Cryptography("invalid encrypted envelope"))?
            .to_be_bytes(),
    );
    for descriptor in descriptors {
        descriptor.encode(&mut output)?;
    }
    if output.len() as u64 > MAX_DIRECTORY_CIPHERTEXT_BYTES {
        return invalid_envelope();
    }
    Ok(output)
}

pub(crate) fn decode_directory(bytes: &[u8]) -> Result<Vec<ObjectDescriptor>> {
    if bytes.len() < 12
        || bytes.len() as u64 > MAX_DIRECTORY_CIPHERTEXT_BYTES
        || bytes[0..8] != DIRECTORY_MAGIC
    {
        return invalid_envelope();
    }
    let count = read_u32(bytes, 8)?;
    if count == 0 || count > MAX_OBJECTS {
        return invalid_envelope();
    }
    let capacity =
        usize::try_from(count).map_err(|_| Error::Cryptography("invalid encrypted envelope"))?;
    let mut descriptors = Vec::with_capacity(capacity);
    let mut offset = 12_usize;
    for _ in 0..count {
        let fixed_end = offset
            .checked_add(28)
            .ok_or(Error::Cryptography("invalid encrypted envelope"))?;
        if fixed_end > bytes.len() || bytes[offset + 1] != 0 || bytes[offset + 27] != 0 {
            return invalid_envelope();
        }
        let object_type = ObjectType::from_code(bytes[offset])?;
        let id_length = usize::from(read_u16(bytes, offset + 2)?);
        if id_length == 0 || id_length > MAX_OBJECT_ID_BYTES {
            return invalid_envelope();
        }
        let id_end = fixed_end
            .checked_add(id_length)
            .ok_or(Error::Cryptography("invalid encrypted envelope"))?;
        if id_end > bytes.len() {
            return invalid_envelope();
        }
        let object_id = std::str::from_utf8(&bytes[fixed_end..id_end])
            .map_err(|_| Error::Cryptography("invalid encrypted envelope"))?
            .to_owned();
        let descriptor = ObjectDescriptor {
            object_type,
            object_id,
            plaintext_length: read_u64(bytes, offset + 4)?,
            ciphertext_length: read_u64(bytes, offset + 12)?,
            stream_nonce: copy_array(bytes, offset + 20)?,
        };
        validate_descriptor(&descriptor)?;
        descriptors.push(descriptor);
        offset = id_end;
    }
    if offset != bytes.len() {
        return invalid_envelope();
    }
    validate_descriptor_set(&descriptors)?;
    Ok(descriptors)
}

pub(crate) fn stream_ciphertext_length(plaintext_length: u64) -> Result<u64> {
    let chunks = if plaintext_length == 0 {
        1
    } else {
        plaintext_length
            .checked_add(u64::from(CHUNK_BYTES) - 1)
            .ok_or(Error::Cryptography("invalid encrypted envelope"))?
            / u64::from(CHUNK_BYTES)
    };
    if chunks > MAX_CHUNKS {
        return invalid_envelope();
    }
    plaintext_length
        .checked_add(
            chunks
                .checked_mul(GCM_TAG_BYTES)
                .ok_or(Error::Cryptography("invalid encrypted envelope"))?,
        )
        .ok_or(Error::Cryptography("invalid encrypted envelope"))
}

fn validate_descriptor(descriptor: &ObjectDescriptor) -> Result<()> {
    if descriptor.object_id.is_empty()
        || descriptor.object_id.len() > MAX_OBJECT_ID_BYTES
        || descriptor.object_id.as_bytes().contains(&0)
        || descriptor.plaintext_length > descriptor.object_type.plaintext_limit()
        || descriptor.ciphertext_length != stream_ciphertext_length(descriptor.plaintext_length)?
    {
        return invalid_envelope();
    }
    Ok(())
}

fn validate_descriptor_set(descriptors: &[ObjectDescriptor]) -> Result<()> {
    if descriptors.is_empty()
        || descriptors.len() > MAX_OBJECTS as usize
        || descriptors[0].object_type != ObjectType::Manifest
        || descriptors
            .iter()
            .filter(|descriptor| descriptor.object_type == ObjectType::Manifest)
            .count()
            != 1
    {
        return invalid_envelope();
    }
    let mut ids = std::collections::HashSet::new();
    for descriptor in descriptors {
        validate_descriptor(descriptor)?;
        if !ids.insert((descriptor.object_type.code(), descriptor.object_id.as_str())) {
            return invalid_envelope();
        }
    }
    Ok(())
}

fn validate_directory_ciphertext_length(length: u64) -> Result<()> {
    if !(GCM_TAG_BYTES..=MAX_DIRECTORY_CIPHERTEXT_BYTES).contains(&length) {
        return invalid_envelope();
    }
    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    Ok(u16::from_be_bytes(copy_array(bytes, offset)?))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_be_bytes(copy_array(bytes, offset)?))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64> {
    Ok(u64::from_be_bytes(copy_array(bytes, offset)?))
}

fn copy_array<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N]> {
    let end = offset
        .checked_add(N)
        .ok_or(Error::Cryptography("invalid encrypted envelope"))?;
    bytes
        .get(offset..end)
        .ok_or(Error::Cryptography("invalid encrypted envelope"))?
        .try_into()
        .map_err(|_| Error::Cryptography("invalid encrypted envelope"))
}

fn invalid_envelope<T>() -> Result<T> {
    Err(Error::Cryptography("invalid encrypted envelope"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(object_type: ObjectType, id: &str, length: u64) -> Result<ObjectDescriptor> {
        Ok(ObjectDescriptor {
            object_type,
            object_id: id.to_owned(),
            plaintext_length: length,
            ciphertext_length: stream_ciphertext_length(length)?,
            stream_nonce: [7; 7],
        })
    }

    #[test]
    fn header_round_trip_and_hostile_parameters() -> Result<()> {
        let header = EncryptedHeader {
            salt: [1; 16],
            wrap_nonce: [2; 12],
            directory_nonce: [3; 7],
            directory_ciphertext_length: 128,
            wrapped_bundle_key: [4; 48],
        };
        let encoded = header.encode()?;
        assert!(EncryptedHeader::decode(&encoded, 264).is_ok());
        for offset in [0, 16, 18, 20, 24, 28, 32, 36, 75] {
            let mut hostile = encoded;
            hostile[offset] ^= 1;
            assert!(EncryptedHeader::decode(&hostile, 264).is_err());
        }
        assert!(EncryptedHeader::decode(&encoded, 200).is_err());
        assert!(EncryptedHeader::decode(&encoded, MAX_ENVELOPE_BYTES + 1).is_err());
        Ok(())
    }

    #[test]
    fn directory_round_trip_and_exact_eof() -> Result<()> {
        let descriptors = vec![
            descriptor(ObjectType::Manifest, "manifest", 1024)?,
            descriptor(ObjectType::Artifact, "art_123", CHUNK_BYTES as u64 + 1)?,
        ];
        let encoded = encode_directory(&descriptors)?;
        assert_eq!(decode_directory(&encoded)?, descriptors);
        let mut trailing = encoded.clone();
        trailing.push(0);
        assert!(decode_directory(&trailing).is_err());
        let mut truncated = encoded;
        truncated.pop();
        assert!(decode_directory(&truncated).is_err());
        Ok(())
    }

    #[test]
    fn directory_rejects_invalid_order_duplicates_and_lengths() -> Result<()> {
        let artifact = descriptor(ObjectType::Artifact, "duplicate", 0)?;
        assert!(encode_directory(std::slice::from_ref(&artifact)).is_err());
        assert!(
            encode_directory(&[
                descriptor(ObjectType::Manifest, "manifest", 1)?,
                descriptor(ObjectType::Manifest, "second", 1)?,
            ])
            .is_err()
        );
        assert!(
            encode_directory(&[
                descriptor(ObjectType::Manifest, "manifest", 1)?,
                artifact.clone(),
                artifact,
            ])
            .is_err()
        );
        let mut wrong_length = descriptor(ObjectType::Manifest, "manifest", 1)?;
        wrong_length.ciphertext_length += 1;
        assert!(encode_directory(&[wrong_length]).is_err());
        Ok(())
    }

    #[test]
    fn stream_lengths_are_checked() -> Result<()> {
        assert_eq!(stream_ciphertext_length(0)?, 16);
        assert_eq!(stream_ciphertext_length(1)?, 17);
        assert_eq!(
            stream_ciphertext_length(CHUNK_BYTES as u64)?,
            CHUNK_BYTES as u64 + 16
        );
        assert_eq!(
            stream_ciphertext_length(CHUNK_BYTES as u64 + 1)?,
            CHUNK_BYTES as u64 + 33
        );
        assert!(stream_ciphertext_length(MAX_ARTIFACT_BYTES + 1).is_err());
        Ok(())
    }

    #[test]
    fn aad_binds_header_and_exact_descriptor() -> Result<()> {
        let header = [9_u8; HEADER_BYTES];
        let first = descriptor(ObjectType::Artifact, "art_one", 10)?;
        let second = descriptor(ObjectType::Artifact, "art_two", 10)?;
        assert_ne!(object_aad(&header, &first)?, object_aad(&header, &second)?);
        let mut changed_header = header;
        changed_header[0] ^= 1;
        assert_ne!(
            object_aad(&header, &first)?,
            object_aad(&changed_header, &first)?
        );
        assert_ne!(directory_aad(&header), object_aad(&header, &first)?);
        Ok(())
    }
}
