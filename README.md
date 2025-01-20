# Commands
cargo build
cargo run -p coordinator
cargo run -p worker
cargo run -p client



# Redis
docker run --name redis -p 6379:6379 -d redis
curl 0.0.0.0:3000/build_task_response -H 'Content-Type: application/json' -d '{"id": 1, "repository": "https://github.com/example/project.git", "branch": "main"}' -X POST
