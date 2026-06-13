use crate::error::AssetError;
use crate::tag_dictionary::TagDictionary;

/// Returns `true` when `tag` is present in the dictionary.
pub fn tag_exists(dictionary: &TagDictionary, tag: &str) -> bool {
    dictionary.tags.iter().any(|known| known == tag)
}

/// Validates that a single tag exists in the dictionary.
pub fn validate_tag(dictionary: &TagDictionary, tag: &str) -> Result<(), AssetError> {
    if tag_exists(dictionary, tag) {
        Ok(())
    } else {
        Err(AssetError::UnknownTag(tag.to_string()))
    }
}

/// Validates that every tag in `tags` exists in the dictionary.
pub fn validate_tags(dictionary: &TagDictionary, tags: &[&str]) -> Result<(), AssetError> {
    for tag in tags {
        validate_tag(dictionary, tag)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag_dictionary::TAG_DICTIONARY_SCHEMA_VERSION;

    fn sample_dictionary() -> TagDictionary {
        TagDictionary {
            schema_version: TAG_DICTIONARY_SCHEMA_VERSION,
            tags: vec!["State.Stunned".to_string(), "Damage.Fire".to_string()],
        }
    }

    #[test]
    fn known_tag_passes_validation() {
        let dictionary = sample_dictionary();
        assert!(tag_exists(&dictionary, "Damage.Fire"));
        validate_tag(&dictionary, "Damage.Fire").expect("known tag");
    }

    #[test]
    fn unknown_tag_fails_validation() {
        let dictionary = sample_dictionary();
        let error = validate_tag(&dictionary, "Ability.Missing").unwrap_err();
        assert!(matches!(error, AssetError::UnknownTag(_)));
    }
}
