# Note Sharing App

This app extends the Loco.rs REST API starter to add note sharing features.

## New Endpoints

1. Share a note: POST /api/notes/:id/share
   - Share a specific note with another user

2. Get shared notes: GET /api/notes/shared
   - Get all notes shared with you

3. Share all notes: POST /api/notes/share-all
   - Share all your notes with another user

## Updated Endpoints

- GET /api/notes: Now returns your notes and notes shared with you
- GET /api/notes/:id: Access a note if you own it or it's shared with you
- POST/PUT /api/notes/:id: Update a note (owner only)
- DELETE /api/notes/:id: Delete a note (owner only)

## Testing

Run tests with:
```
cargo test
```
