#[cfg(feature = "web")]
pub mod scanner {
    use chrono::NaiveDate;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    use crate::category::{suggest_category, suggest_expiration_date};

    #[derive(Debug, Error)]
    pub enum ReceiptError {
        #[error("Anthropic API key not configured")]
        NoApiKey,
        #[error("HTTP request failed: {0}")]
        Http(#[from] reqwest::Error),
        #[error("API error: {0}")]
        ApiError(String),
        #[error("Failed to parse receipt items: {0}")]
        ParseError(String),
        #[error("Image too large (max 5MB)")]
        ImageTooLarge,
        #[error("Unsupported image type: {0}")]
        UnsupportedType(String),
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct ScannedItem {
        pub name: String,
        pub quantity: u32,
        pub unit: String,
        pub category: Option<String>,
        pub expiration_date: Option<NaiveDate>,
    }

    // Intermediate struct for deserialization that accepts fractional quantities
    #[derive(Debug, Deserialize)]
    struct RawScannedItem {
        name: String,
        #[serde(default = "default_quantity")]
        quantity: f64,
        #[serde(default = "default_unit")]
        unit: String,
    }

    impl RawScannedItem {
        fn into_scanned_item(self) -> ScannedItem {
            // For fractional kg/lb quantities, convert to grams/oz for better precision
            let (quantity, unit) = if self.quantity.fract() != 0.0 {
                match self.unit.as_str() {
                    "kg" => ((self.quantity * 1000.0).round() as u32, "g".to_string()),
                    "lb" | "lbs" => ((self.quantity * 16.0).round() as u32, "oz".to_string()),
                    _ => (self.quantity.ceil() as u32, self.unit),
                }
            } else {
                (self.quantity as u32, self.unit)
            };
            ScannedItem {
                name: self.name,
                quantity: quantity.max(1),
                unit,
                category: None,
                expiration_date: None,
            }
        }
    }

    fn default_quantity() -> f64 {
        1.0
    }

    fn default_unit() -> String {
        "count".to_string()
    }

    const MAX_IMAGE_SIZE: usize = 5 * 1024 * 1024; // 5MB

    const SYSTEM_PROMPT: &str = r#"You are a grocery receipt parser. Extract every grocery item from the receipt image.

For each item, return a JSON object with:
- "name": The GENERIC grocery item name, normalized to how a person would say it. Strip brand names, store codes, abbreviations, and product numbers. Examples:
  - "BORDEN WHL MLK 1GAL" → "Whole Milk"
  - "GV 2% RDCD FAT MLK" → "2% Milk"
  - "BNLS SKNLS CHKN BRST" → "Chicken Breast"
  - "ORG BNS GRN CUT" → "Green Beans"
  - "DOLE BANANA" → "Bananas"
  - "KROGER SHRD MZZRLA" → "Shredded Mozzarella"
  - "HNY CRISP APL" → "Honeycrisp Apples"
  - "LG EGGS GRD A" → "Eggs"
  - "WHL WHT BRD 20OZ" → "Whole Wheat Bread"
  Keep useful qualifiers (whole wheat, 2%, organic, etc.) but drop brand names and size codes.
- "quantity": The quantity purchased as a number (can be fractional for weighted items, default 1 if unclear)
- "unit": The unit of measurement. Use the weight/volume from the receipt if shown (kg, lb, oz, gallon, etc.), otherwise use "count"

Rules:
- Do NOT include non-grocery lines (tax, totals, subtotals, store info, coupons, discounts, change, payment method, bag fees, loyalty savings)
- Do NOT include duplicate entries for the same item
- If multiple quantities of the same item appear, combine them into one entry
- If you cannot read an item clearly, skip it rather than guessing
- For produce sold by weight, use the weight shown on the receipt as the quantity

Return ONLY a JSON array. No markdown fencing, no explanation, just the array."#;

    pub fn validate_image(bytes: &[u8], content_type: &str) -> Result<(), ReceiptError> {
        if bytes.len() > MAX_IMAGE_SIZE {
            return Err(ReceiptError::ImageTooLarge);
        }
        match content_type {
            "image/jpeg" | "image/png" | "image/webp" | "image/gif" => Ok(()),
            other => Err(ReceiptError::UnsupportedType(other.to_string())),
        }
    }

    pub async fn scan_receipt(
        api_key: &str,
        image_bytes: &[u8],
        media_type: &str,
    ) -> Result<Vec<ScannedItem>, ReceiptError> {
        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, image_bytes);

        let request_body = serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 4096,
            "system": SYSTEM_PROMPT,
            "messages": [{
                "role": "user",
                "content": [
                    {
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": media_type,
                            "data": b64,
                        }
                    },
                    {
                        "type": "text",
                        "text": "Extract all grocery items from this receipt."
                    }
                ]
            }]
        });

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .timeout(std::time::Duration::from_secs(30))
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ReceiptError::ApiError(format!("{status}: {body}")));
        }

        let resp_body: serde_json::Value = response.json().await?;

        let text = resp_body["content"]
            .as_array()
            .and_then(|blocks| {
                blocks
                    .iter()
                    .find(|b| b["type"] == "text")
                    .and_then(|b| b["text"].as_str())
            })
            .ok_or_else(|| ReceiptError::ParseError("no text in response".to_string()))?;

        // Extract JSON array from response, handling markdown fences and surrounding text
        let json_str = extract_json_array(text);

        let raw_items: Vec<RawScannedItem> = serde_json::from_str(json_str)
            .map_err(|e| ReceiptError::ParseError(format!("{e}: {json_str}")))?;
        let mut items: Vec<ScannedItem> = raw_items
            .into_iter()
            .map(|r| r.into_scanned_item())
            .collect();

        // Enrich with local category and expiration suggestions
        for item in &mut items {
            if item.category.is_none() {
                item.category = suggest_category(&item.name).map(String::from);
            }
            if item.expiration_date.is_none() {
                item.expiration_date = suggest_expiration_date(&item.name);
            }
        }

        Ok(items)
    }

    fn extract_json_array(text: &str) -> &str {
        let text = text.trim();

        // Try stripping markdown code fences first
        if let Some(inner) = text
            .strip_prefix("```json")
            .or_else(|| text.strip_prefix("```"))
            .and_then(|s| s.strip_suffix("```"))
        {
            return inner.trim();
        }

        // Find the JSON array within surrounding text
        if let Some(start) = text.find('[') {
            if let Some(end) = text.rfind(']') {
                if end > start {
                    return &text[start..=end];
                }
            }
        }

        text
    }
}
