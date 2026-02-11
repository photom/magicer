# Test Plan: WindowsCompatibleFilename

## test_filename_valid_accepted

**Setup:**
- Valid filename string "test.txt"

**Execution:**
- Call `WindowsCompatibleFilename::new("test.txt")`

**Assertions:**
- Result is `Ok`
- `as_str()` returns "test.txt"

## test_filename_with_slash_rejected

**Setup:**
- Filename string containing forward slash "folder/file.txt"

**Execution:**
- Call `WindowsCompatibleFilename::new("folder/file.txt")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::InvalidCharacter`

## test_filename_with_null_byte_rejected

**Setup:**
- Filename string containing null byte "file\0.txt"

**Execution:**
- Call `WindowsCompatibleFilename::new("file\0.txt")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::InvalidCharacter`

## test_filename_too_long_rejected

**Setup:**
- Filename string exceeding 310 characters

**Execution:**
- Call `WindowsCompatibleFilename::new(long_string)`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::ExceedsMaxLength`

## test_filename_max_length_accepted

**Setup:**
- Filename string exactly 310 characters

**Execution:**
- Call `WindowsCompatibleFilename::new(max_string)`

**Assertions:**
- Result is `Ok`

## test_filename_empty_rejected

**Setup:**
- Empty string ""

**Execution:**
- Call `WindowsCompatibleFilename::new("")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::EmptyValue`

## test_filename_unicode_accepted

**Setup:**
- Unicode filename "файл_测试_🎉.txt"

**Execution:**
- Call `WindowsCompatibleFilename::new("файл_测试_🎉.txt")`

**Assertions:**
- Result is `Ok`
- `as_str()` returns original string

# Test Plan: RelativePath

## test_path_valid_accepted

**Setup:**
- Valid relative path "uploads/file.txt"

**Execution:**
- Call `RelativePath::new("uploads/file.txt")`

**Assertions:**
- Result is `Ok`
- `as_str()` returns "uploads/file.txt"

## test_path_absolute_rejected

**Setup:**
- Absolute path "/etc/passwd"

**Execution:**
- Call `RelativePath::new("/etc/passwd")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::AbsolutePath`

## test_path_traversal_rejected

**Setup:**
- Path with traversal "../etc/passwd"

**Execution:**
- Call `RelativePath::new("../etc/passwd")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::PathTraversal`

## test_path_double_slash_rejected

**Setup:**
- Path with double slash "data//file.txt"

**Execution:**
- Call `RelativePath::new("data//file.txt")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::InvalidPath`

## test_path_ends_with_dot_rejected

**Setup:**
- Path ending with dot "data/."

**Execution:**
- Call `RelativePath::new("data/.")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::InvalidPath`

## test_path_leading_space_rejected

**Setup:**
- Path with leading space " data/file.txt"

**Execution:**
- Call `RelativePath::new(" data/file.txt")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::InvalidPath`

# Test Plan: RequestId

## test_request_id_generate

**Setup:**
- None

**Execution:**
- Call `RequestId::generate()`

**Assertions:**
- Returns a `RequestId`
- `as_str()` returns a valid UUID v4 string

## test_request_id_from_valid_uuid

**Setup:**
- Valid UUID v4 string

**Execution:**
- Call `RequestId::try_from(valid_uuid)`

**Assertions:**
- Result is `Ok`

## test_request_id_from_invalid_uuid

**Setup:**
- Invalid UUID string "not-a-uuid"

**Execution:**
- Call `RequestId::try_from("not-a-uuid")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::InvalidCharacter`

# Test Plan: MimeType

## test_mime_type_valid_accepted

**Setup:**
- Valid MIME type string "application/pdf"

**Execution:**
- Call `MimeType::try_from("application/pdf")`

**Assertions:**
- Result is `Ok`
- `as_str()` returns "application/pdf"

## test_mime_type_invalid_format_rejected

**Setup:**
- Invalid MIME type string "not-a-mime" (missing slash)

**Execution:**
- Call `MimeType::try_from("not-a-mime")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::InvalidCharacter`

## test_mime_type_empty_rejected

**Setup:**
- Empty string ""

**Execution:**
- Call `MimeType::try_from("")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::EmptyValue`

# Test Plan: BasicAuthCredentials

## test_credentials_valid_accepted

**Setup:**
- Username "user", password "pass"

**Execution:**
- Call `BasicAuthCredentials::new("user", "pass")`

**Assertions:**
- Result is `Ok`
- `username()` returns "user"
- `password()` returns "pass"

## test_credentials_empty_username_rejected

**Setup:**
- Username "", password "pass"

**Execution:**
- Call `BasicAuthCredentials::new("", "pass")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::EmptyValue`

## test_credentials_empty_password_rejected

**Setup:**
- Username "user", password ""

**Execution:**
- Call `BasicAuthCredentials::new("user", "")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::EmptyValue`
