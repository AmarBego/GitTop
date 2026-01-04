# GitTop Test Specs

Test fixtures and mock data for development and testing.

## Usage

Run with mock notifications to test scroll performance:

```powershell
# Run with 1000 mock notifications (from project root)
cargo run -- --mock-notifications 1000

# Or better yet
bacon run mock
```

## Mock Data Files

- `mock_notifications.rs` - Mock notification generator for scroll testing
