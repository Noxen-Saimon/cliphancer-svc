use url::Url;

/// List of lowercase tracking query parameters to be stripped.
const TRACKING_PARAMS: &[&str] = &[
    // Google Analytics / Ads
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "utm_id",
    "utm_source_platform",
    "utm_creative_format",
    "utm_marketing_tactic",
    "gclid",
    "gbraid",
    "wbraid",
    "gad_source",
    "gclsrc",
    "dclid",
    // Meta (Facebook, Instagram)
    "fbclid",
    "igshid",
    "igsh",
    // YouTube
    "si",
    "feature",
    "pp",
    // Twitter / X
    "ref_src",
    "ref_url",
    "twclid",
    // TikTok
    "_r",
    "_t",
    "_s",
    // Microsoft / Bing
    "msclkid",
    // Yandex
    "yclid",
    // Mailchimp & Email marketing
    "mc_cid",
    "mc_eid",
    "vero_id",
    "vero_conv",
    "wickedid",
    // E-commerce (AliExpress, Shopee, Lazada, eBay)
    "spm",
    "scm",
    "mkevt",
    "mkcid",
    "mkrid",
    "campid",
    "toolid",
    "customid",
    // Matomo / Piwik
    "pk_campaign",
    "pk_kwd",
    "pk_source",
    "pk_medium",
    "piwik_campaign",
    "piwik_kwd",
    // HubSpot
    "hsa_cam",
    "hsa_grp",
    "hsa_mt",
    "hsa_src",
    "hsa_ad",
    "hsa_acc",
    "hsa_net",
    "hsa_kw",
    "hsa_tgt",
    "hsa_ver",
];

/// Checks if a query key matches any known tracking parameter.
fn is_tracking_param(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    TRACKING_PARAMS.contains(&lower.as_str())
}

/// Cleans a single URL string by stripping tracking query parameters.
/// Returns Some(cleaned_url) if tracking parameters were stripped, or None if unchanged.
pub fn clean_url_str(raw: &str) -> Option<String> {
    let mut parsed = Url::parse(raw).ok()?;

    if parsed.query().is_none() {
        return None;
    }

    let mut retained_pairs = Vec::new();
    let mut modified = false;

    for (k, v) in parsed.query_pairs() {
        if is_tracking_param(&k) {
            modified = true;
        } else {
            retained_pairs.push((k.into_owned(), v.into_owned()));
        }
    }

    if !modified {
        return None;
    }

    if retained_pairs.is_empty() {
        parsed.set_query(None);
    } else {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        for (k, v) in retained_pairs {
            serializer.append_pair(&k, &v);
        }
        let new_query = serializer.finish();
        parsed.set_query(Some(&new_query));
    }

    Some(parsed.to_string())
}

/// Fast-path single URL processor:
/// - Only triggers if the copied clipboard content is strictly a SINGLE URL (starts with http:// or https:// and contains no whitespace).
/// - Completely ignores regular text, documents, code, or paragraphs containing multiple links.
/// - Strips tracking query parameters and trims leading/trailing whitespace.
pub fn clean_text(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Fast reject: must start with http:// or https://
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return None;
    }

    // Fast reject: a single link cannot contain internal whitespace or newlines
    if trimmed.contains(|c: char| c.is_whitespace()) {
        return None;
    }

    // Process tracking parameters on the single URL
    if let Some(cleaned) = clean_url_str(trimmed) {
        Some(cleaned)
    } else if trimmed != input {
        // The URL had leading/trailing whitespace but no tracking parameters
        Some(trimmed.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_youtube_url_stripped() {
        let input = "https://youtu.be/dQw4w9WgXcQ?si=abcdef123456";
        let cleaned = clean_text(input).expect("Should clean single YouTube URL");
        assert_eq!(cleaned, "https://youtu.be/dQw4w9WgXcQ");
    }

    #[test]
    fn test_single_utm_url_stripped_and_keep_others() {
        let input = "https://example.com/product?id=999&utm_source=twitter&utm_medium=social";
        let cleaned = clean_text(input).expect("Should clean single UTM params");
        assert_eq!(cleaned, "https://example.com/product?id=999");
    }

    #[test]
    fn test_single_url_with_whitespace_trimmed() {
        let input = "   https://twitter.com/status/123?ref_src=twsrc   \n";
        let cleaned = clean_text(input).expect("Should clean and trim single URL");
        assert_eq!(cleaned, "https://twitter.com/status/123");
    }

    #[test]
    fn test_ignore_paragraphs_or_sentences_with_links() {
        // Sentences with links or multiple links should be ignored to stay super lightweight
        let sentence = "Lihat video ini: https://www.youtube.com/watch?v=123&si=test! Keren kan?";
        assert_eq!(clean_text(sentence), None);

        let multi_links = "https://example.com/one?utm_source=a\nhttps://example.com/two?utm_source=b";
        assert_eq!(clean_text(multi_links), None);
    }

    #[test]
    fn test_ignore_normal_text_and_code() {
        let regular_text = "Halo dunia! Ini teks biasa tanpa link.";
        assert_eq!(clean_text(regular_text), None);

        let code = "fn main() { println!(\"https://not_a_single_link\"); }";
        assert_eq!(clean_text(code), None);
    }
}
