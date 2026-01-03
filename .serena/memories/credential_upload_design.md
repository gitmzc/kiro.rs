# Credential Upload Feature - Interaction Design

## 1. Upload Component
- **Drag & Drop Area**: Large, dashed-border zone centered in a modal or page section. Text: "Drag credential JSON here or click to upload".
- **File Input**: Hidden file input triggered by clicking the zone.
- **Batch Support**: Allow selecting multiple files or dragging multiple files at once.

## 2. Pre-upload Validation & Preview
- **JSON Parsing**: Parse client-side immediately upon file selection.
- **Validation**: Check for required fields (`refresh_token`, `auth_method`). Show error if invalid JSON or missing fields.
- **Preview Modal/Card**:
    - **Header**: Filename (e.g., `credentials.json`).
    - **Key Info**: Display `auth_method` (e.g., "builder-id"), `email` (if present), and `expires_at` (formatted nicely).
    - **Token Preview**: Show first/last 4 chars of `refresh_token` (e.g., `ey...a5`).
    - **Raw View**: Collapsible "View Raw JSON" section.

## 3. Field Mapping Display
- Highlight which fields kiro-rs will actually use vs. ignore.
- **Used**: `refresh_token`, `auth_method`, `client_id`, `client_secret`, `expires_at`.
- **Ignored/Informational**: `email`, `type`, `access_token` (usually refreshed immediately).

## 4. Feedback Design
- **Success**: Toast notification "Credential imported successfully". The new credential appears instantly in the list with a "New" badge.
- **Failure**:
    - **Validation Error**: "Invalid JSON format" or "Missing 'refresh_token'".
    - **Duplicate**: "Credential already exists".
    - **Server Error**: "Failed to save credential".

## 5. Batch Upload Strategy
- **Queue System**: If multiple files are dropped, list them all in a "Pending Upload" list.
- **Individual Status**: Show a spinner/check/cross icon next to each file as they are processed sequentially or in parallel.
- **Summary**: "3 files imported, 1 failed."
