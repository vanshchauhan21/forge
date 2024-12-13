---
name: todo-app
---

# Specifications

- Create a simple TODO application and expose REST APIs to interact with it.
- I should be able to create a TODO item, update it, delete it, and list all the TODO items.
- The List API should support pagination.
- I should be able to filter the TODO items based on the `done` field.
- I should be able to undo and redo the last action.
- It should be able to mark a TODO item as done.
- API Endpoints:
  - `POST /todos` - Create a TODO item
  - `GET /todos` - List all TODO items (pagination supported)
  - `GET /todos/{id}` - Get a TODO item by ID
  - `PUT /todos/{id}` - Update a TODO item by ID
  - `DELETE /todos/{id}` - Delete a TODO item by ID
  - `POST /todos/{id}/done` - Mark a TODO item as done
  - `POST /todos/undo` - Undo the last action
  - `POST /todos/redo` - Redo the last action
  - `GET /todos/history` - Get the history of actions (pagination supported)

# Technical Requirements

- It should be written in Rust
- Use Axum as the web framework
- The backend should be written in surrealDB
- Add unit tests for the backend
