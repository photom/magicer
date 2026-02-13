# Test Plan: WindowsCompatibleFilename

## test_new_with_valid_name_returns_success

**Setup:**
- Valid filename string "test.txt"

**Execution:**
- Call `WindowsCompatibleFilename::new("test.txt")`

**Assertions:**
- Result is `Ok`
- `as_str()` returns "test.txt"

## test_new_with_slash_returns_error

**Setup:**
- Filename string containing forward slash "folder/file.txt"

**Execution:**
- Call `WindowsCompatibleFilename::new("folder/file.txt")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::InvalidCharacter`

## test_new_with_null_byte_returns_error

**Setup:**
- Filename string containing null byte "file\0.txt"

**Execution:**
- Call `WindowsCompatibleFilename::new("file\0.txt")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::InvalidCharacter`

## test_new_with_too_long_name_returns_error

**Setup:**
- Filename string exceeding 310 characters

**Execution:**
- Call `WindowsCompatibleFilename::new(long_string)`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::ExceedsMaxLength`

## test_new_with_max_length_name_returns_success

**Setup:**
- Filename string exactly 310 characters

**Execution:**
- Call `WindowsCompatibleFilename::new(max_string)`

**Assertions:**
- Result is `Ok`

## test_new_with_empty_name_returns_error

**Setup:**
- Empty string ""

**Execution:**
- Call `WindowsCompatibleFilename::new("")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::EmptyValue`

## test_new_with_unicode_name_returns_success

**Setup:**
- Unicode filename "файл_测试_🎉.txt"

**Execution:**
- Call `WindowsCompatibleFilename::new("файл_测试_🎉.txt")`

**Assertions:**
- Result is `Ok`
- `as_str()` returns original string

## test_new_with_windows_reserved_characters_returns_success

**Setup:**
- Filename containing Windows reserved characters (\, :, *, ?, ", <, >, |)

**Execution:**
- Call `WindowsCompatibleFilename::new(reserved_name)`

**Assertions:**
- Result is `Ok`

# Test Plan: RelativePath

## test_new_with_valid_path_returns_success

**Setup:**
- Valid relative path "uploads/file.txt"

**Execution:**
- Call `RelativePath::new("uploads/file.txt")`

**Assertions:**
- Result is `Ok`
- `as_str()` returns "uploads/file.txt"

## test_new_with_absolute_path_returns_error

**Setup:**
- Absolute path "/etc/passwd"

**Execution:**
- Call `RelativePath::new("/etc/passwd")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::AbsolutePath`

## test_new_with_traversal_returns_error

**Setup:**
- Path with traversal "../etc/passwd", "data/../file.txt"

**Execution:**
- Call `RelativePath::new(path)`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::PathTraversal`

## test_new_with_double_slash_returns_error

**Setup:**
- Path with double slash "data//file.txt"

**Execution:**
- Call `RelativePath::new("data//file.txt")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::InvalidPath`

## test_new_with_dot_suffix_returns_error

**Setup:**
- Path ending with dot "data/."

**Execution:**
- Call `RelativePath::new("data/.")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::InvalidPath`

## test_new_with_leading_space_returns_error

**Setup:**
- Path with leading space " data/file.txt"

**Execution:**
- Call `RelativePath::new(" data/file.txt")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::InvalidPath`

# Test Plan: RequestId

## test_generate_returns_valid_uuid_v4

**Setup:**
- None

**Execution:**
- Call `RequestId::generate()`

**Assertions:**
- Returns a `RequestId`
- `as_str()` returns a valid UUID v4 string

## test_try_from_with_valid_uuid_returns_success

**Setup:**
- Valid UUID v4 string

**Execution:**
- Call `RequestId::try_from(valid_uuid)`

**Assertions:**
- Result is `Ok`

## test_try_from_with_invalid_uuid_returns_error

**Setup:**
- Invalid UUID string "not-a-uuid"

**Execution:**
- Call `RequestId::try_from("not-a-uuid")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::InvalidCharacter`

# Test Plan: MimeType

## test_try_from_with_valid_mime_returns_success

**Setup:**
- Valid MIME type string "application/pdf"

**Execution:**
- Call `MimeType::try_from("application/pdf")`

**Assertions:**
- Result is `Ok`
- `as_str()` returns "application/pdf"

## test_try_from_with_invalid_format_returns_error

**Setup:**
- Invalid MIME type string "not-a-mime" (missing slash)

**Execution:**
- Call `MimeType::try_from("not-a-mime")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::InvalidCharacter`

## test_try_from_with_empty_string_returns_error

**Setup:**
- Empty string ""

**Execution:**
- Call `MimeType::try_from("")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::EmptyValue`

# Test Plan: BasicAuthCredentials

## test_new_with_valid_credentials_returns_success

**Setup:**
- Username "user", password "pass"

**Execution:**
- Call `BasicAuthCredentials::new("user", "pass")`

**Assertions:**
- Result is `Ok`
- `username()` returns "user"
- `password()` returns "pass"

## test_new_with_empty_username_returns_error

**Setup:**
- Username "", password "pass"

**Execution:**
- Call `BasicAuthCredentials::new("", "pass")`

**Assertions:**
- Result is `Err`
- Error variant matches `ValidationError::EmptyValue`

## test_verify_with_correct_credentials_returns_true

**Setup:**
- Username "admin", password "secret"

**Execution:**
- Call `BasicAuthCredentials::new("admin", "secret")`
- Call `creds.verify("admin", "secret")`

**Assertions:**
- `verify` returns `true`
- `verify("admin", "wrong")` returns `false`
