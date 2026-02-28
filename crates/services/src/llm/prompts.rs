/// Build the system prompt based on the document type.
pub fn system_prompt(document_type: &str) -> String {
    match document_type {
        "invoice" => SYSTEM_INVOICE.to_string(),
        "bank_statement" => SYSTEM_BANK_STATEMENT.to_string(),
        "payment" => SYSTEM_PAYMENT.to_string(),
        "receipt" => SYSTEM_RECEIPT.to_string(),
        _ => SYSTEM_GENERIC.to_string(),
    }
}

/// Build the user prompt for a vision model processing a document page image.
pub fn vision_user_prompt(document_type: &str, page_num: usize, total_pages: usize) -> String {
    let doc_instruction = match document_type {
        "invoice" => "Extract the invoice data from this document image.",
        "bank_statement" => "Extract the bank statement data from this document image.",
        "payment" => "Extract the payment data from this document image.",
        "receipt" => "Extract the receipt data from this document image.",
        _ => "Extract all key information from this document image.",
    };

    if total_pages > 1 {
        format!(
            "{doc_instruction}\n\nThis is page {page_num} of {total_pages}. \
             Extract all visible data from this page. \
             Respond with a single JSON object only. No explanations."
        )
    } else {
        format!(
            "{doc_instruction}\n\nRespond with a single JSON object only. No explanations."
        )
    }
}

/// Build the merge prompt for combining per-page extraction results.
pub fn merge_pages_prompt(
    document_type: &str,
    page_results: &[serde_json::Value],
) -> String {
    let pages_json: Vec<String> = page_results
        .iter()
        .enumerate()
        .map(|(i, v)| format!("Page {}:\n{}", i + 1, serde_json::to_string_pretty(v).unwrap_or_default()))
        .collect();

    let doc_label = match document_type {
        "invoice" => "invoice",
        "bank_statement" => "bank statement",
        "payment" => "payment document",
        "receipt" => "receipt",
        _ => "document",
    };

    format!(
        "The following JSON objects were extracted from individual pages of a {doc_label}. \
         Merge them into a single coherent JSON object.\n\n\
         Rules:\n\
         - Combine line_items/transactions/items from all pages into one array\n\
         - Use header fields (vendor, dates, totals) from whichever page has them\n\
         - If totals appear on multiple pages, prefer the final page's values\n\
         - Remove duplicates\n\
         - Keep the same JSON schema as the individual pages\n\n\
         {}\n\n\
         Respond with a single merged JSON object only. No explanations.",
        pages_json.join("\n\n")
    )
}

/// Build the user prompt with the document text.
pub fn user_prompt(raw_text: &str, document_type: &str) -> String {
    let instruction = match document_type {
        "invoice" => "Extract the invoice data from the following document text.",
        "bank_statement" => "Extract the bank statement data from the following document text.",
        "payment" => "Extract the payment data from the following document text.",
        "receipt" => "Extract the receipt data from the following document text.",
        _ => "Extract all key information from the following document text.",
    };

    format!(
        "{instruction}\n\n---\nDOCUMENT TEXT:\n---\n{raw_text}\n---\n\nRespond with a single JSON object only. No explanations."
    )
}

const SYSTEM_INVOICE: &str = r#"You are a document extraction assistant. Extract structured data from invoice documents and return valid JSON.

Return a JSON object with these fields:
{
  "document_type": "invoice",
  "vendor_name": "string",
  "vendor_address": "string or null",
  "vendor_tax_id": "string or null",
  "buyer_name": "string or null",
  "buyer_address": "string or null",
  "buyer_tax_id": "string or null",
  "invoice_number": "string",
  "invoice_date": "YYYY-MM-DD",
  "due_date": "YYYY-MM-DD or null",
  "currency": "3-letter code",
  "subtotal": number or null,
  "tax_amount": number or null,
  "tax_rate": "percentage string or null",
  "total_amount": number,
  "line_items": [
    {
      "description": "string",
      "quantity": number or null,
      "unit_price": number or null,
      "amount": number
    }
  ],
  "payment_terms": "string or null",
  "notes": "string or null",
  "confidence": 0.0-1.0
}

Rules:
- Use null for fields you cannot determine
- Dates in YYYY-MM-DD format
- Amounts as numbers (not strings)
- confidence: your certainty about the extraction accuracy (0.0 to 1.0)
- Return ONLY the JSON object, no markdown, no explanations"#;

const SYSTEM_BANK_STATEMENT: &str = r#"You are a document extraction assistant. Extract structured data from bank statements and return valid JSON.

Return a JSON object with these fields:
{
  "document_type": "bank_statement",
  "bank_name": "string",
  "account_holder": "string or null",
  "account_number": "string (last 4 digits only for security)",
  "statement_period_start": "YYYY-MM-DD",
  "statement_period_end": "YYYY-MM-DD",
  "currency": "3-letter code",
  "opening_balance": number,
  "closing_balance": number,
  "total_deposits": number or null,
  "total_withdrawals": number or null,
  "transactions": [
    {
      "date": "YYYY-MM-DD",
      "description": "string",
      "amount": number,
      "type": "credit" or "debit",
      "balance": number or null
    }
  ],
  "confidence": 0.0-1.0
}

Rules:
- Use null for fields you cannot determine
- Dates in YYYY-MM-DD format
- Amounts as numbers (positive for credits, negative for debits in the amount field)
- Only include last 4 digits of account numbers
- confidence: your certainty about the extraction accuracy (0.0 to 1.0)
- Return ONLY the JSON object, no markdown, no explanations"#;

const SYSTEM_PAYMENT: &str = r#"You are a document extraction assistant. Extract structured data from payment documents and return valid JSON.

Return a JSON object with these fields:
{
  "document_type": "payment",
  "payer_name": "string",
  "payee_name": "string",
  "payment_date": "YYYY-MM-DD",
  "payment_method": "string (bank_transfer, credit_card, cash, check, etc.)",
  "reference_number": "string or null",
  "invoice_reference": "string or null",
  "currency": "3-letter code",
  "amount": number,
  "status": "completed, pending, or failed",
  "notes": "string or null",
  "confidence": 0.0-1.0
}

Rules:
- Use null for fields you cannot determine
- Dates in YYYY-MM-DD format
- Amounts as numbers (not strings)
- confidence: your certainty about the extraction accuracy (0.0 to 1.0)
- Return ONLY the JSON object, no markdown, no explanations"#;

const SYSTEM_RECEIPT: &str = r#"You are a document extraction assistant. Extract structured data from receipts and return valid JSON.

Return a JSON object with these fields:
{
  "document_type": "receipt",
  "merchant_name": "string",
  "merchant_address": "string or null",
  "receipt_number": "string or null",
  "date": "YYYY-MM-DD",
  "time": "HH:MM or null",
  "currency": "3-letter code",
  "items": [
    {
      "description": "string",
      "quantity": number or null,
      "unit_price": number or null,
      "amount": number
    }
  ],
  "subtotal": number or null,
  "tax_amount": number or null,
  "total_amount": number,
  "payment_method": "string or null",
  "confidence": 0.0-1.0
}

Rules:
- Use null for fields you cannot determine
- Dates in YYYY-MM-DD format
- Amounts as numbers (not strings)
- confidence: your certainty about the extraction accuracy (0.0 to 1.0)
- Return ONLY the JSON object, no markdown, no explanations"#;

const SYSTEM_GENERIC: &str = r#"You are a document extraction assistant. Extract all key structured data from documents and return valid JSON.

Analyze the document and determine its type, then extract relevant fields.

Return a JSON object with these fields:
{
  "document_type": "string (invoice, bank_statement, payment, receipt, contract, report, letter, or other)",
  "title": "string or null",
  "date": "YYYY-MM-DD or null",
  "parties": ["list of people/organizations mentioned"],
  "amounts": [
    {
      "label": "string",
      "value": number,
      "currency": "3-letter code or null"
    }
  ],
  "key_fields": {
    "field_name": "value"
  },
  "summary": "brief one-sentence summary of the document",
  "confidence": 0.0-1.0
}

Rules:
- Use null for fields you cannot determine
- Dates in YYYY-MM-DD format
- Amounts as numbers (not strings)
- key_fields: extract any important name-value pairs not covered above
- confidence: your certainty about the extraction accuracy (0.0 to 1.0)
- Return ONLY the JSON object, no markdown, no explanations"#;
