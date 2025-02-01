# Commands
cargo build
cargo run -p coordinator
cargo run -p worker
cargo run -p client



# Redis
docker run --name redis -p 6379:6379 -d redis
curl 0.0.0.0:3000/build_task_response -H 'Content-Type: application/json' -d '{"id": 1, "repository": "https://github.com/example/project.git", "branch": "main"}' -X POST

docker build -t coordinator .
docker build -t worker1 .
docker build -t worker2 .
docker network create cicd-network
docker run -d --name coordinator --network cicd-network -p 3000:3000 coordinator
docker run -d --name worker1 --network cicd-network -p 5001:5001 -e WORKER_PORT=5001 worker
docker run -d --name worker2 --network cicd-network -p 5002:5002 -e WORKER_PORT=5002 worker
docker logs -f worker1
docker logs -f worker2
docker logs -f coordinator


curl -X POST http://localhost:3000/build_task \
     -H "Content-Type: application/json" \
     -d '{"id": 1, "repository": "https://github.com/example.git", "branch": "main"}'

cargo run -- --task "test"
