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

## Design Decisions and Considerations

### Share All Notes Endpoint

A separate endpoint (`POST /api/notes/share-all`) was implemented for sharing all notes at once.

Reasons:
1. Performance: Allows sharing all notes in a single request, reducing network overhead.
2. Frontend Simplicity: Simplifies client-side logic by avoiding multiple API calls (getting note IDs, then sharing each individually).

Alternative Approach:
- Implement an endpoint to share multiple notes in one request, allowing more flexibility than sharing all or one:
  `POST /api/notes/share-multiple`
  ```json
  {
    "note_ids": [1, 2, 3],
    "shared_with_user_id": 4
  }
  ```
  This would balance flexibility and efficiency, suitable for sharing specific sets of notes.

### Access Control Mechanism for Shared Notes

The current implementation uses a `note_shares` table to manage access to shared notes.

How it works:
1. When a note is shared, an entry is created in `note_shares` linking the note to the shared user.
2. When accessing a note, the system checks if the user is either the owner or has a share entry.

Reasons for this approach:
1. Simple to implement and understand.
2. Efficient for checking permissions on individual notes.
3. Allows for easy extension of sharing features (e.g., different permission levels)

Potential improvements:
1. Implement read-only vs. edit permissions for shared notes.
2. Add a mechanism to easily revoke sharing access.
3. Implement time-based or conditional sharing (e.g., share expires after X days).

Alternative approaches:
1. Store an array of shared user IDs directly in the notes table.
   - Pro: Simpler data model.
   - Con: Less flexible for advanced permissions, potentially less efficient for querying.
2. Use a more complex role-based access control system.
   - Pro: More fine-grained control over permissions.
   - Con: Increased complexity in implementation and maintenance.

## Auth

Use a JWT token in the Authorization header for all requests:
Authorization: Bearer your_token_here

## Testing

Run tests with:
```
cargo test
```






 if Eddie has time to do it, could you please ask him to extend the readme with his design decision and other considerations for improvements / alternatives

E.g. access control mechanism of shared notes, why it has been implemented in such way, how could it be done differently, etc. 

E.g. access to all notes, reasoning of the separate endpoint