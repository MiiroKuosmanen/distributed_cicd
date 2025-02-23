# Commands
cargo build
cargo run -p coordinator
cargo run -p worker
cargo run -p client



# Redis
docker run --name redis -p 6379:6379 -d redis
curl 0.0.0.0:3000/build_task_response -H 'Content-Type: application/json' -d '{"id": 1, "repository": "https://github.com/example/project.git", "branch": "main"}' -X POST


# Local development and testing
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


docker stop $(docker ps -a -q)

# Apply kubernetes related stuff
minikube start
minikube addons enable ingress
kubectl create namespace cicd

kubectl apply -f coordinator-deployment.yml
kubectl apply -f worker-deployment.yml
kubectl apply -f worker-hpa.yml
kubectl apply -f coordinator-ingress.yml
kubectl apply -f coordinator-service.yml
kubectl apply -f monitoring-namespace.yml
kubectl apply -f prometheus-deployment.yml
kubectl apply -f grafana-deployment.yml
# Get kubernetes related things and test
kubectl get pods -n cicd
kubectl get svc -n cicd
kubectl get hpa -n cicd
kubectl exec -it $(kubectl get pod -l app=coordinator -n cicd -o jsonpath="{.items[0].metadata.name}") -n cicd -- curl http://worker:5001/execute_task
kubectl logs -l app=worker -n cicd -f
kubectl port-forward -n monitoring svc/prometheus 9090:9090
kubectl port-forward -n monitoring svc/grafana 3100:3100
kubectl rollout restart deployment/coordinator -n cicd
kubectl rollout restart deployment/worker -n cicd

# Expose minikube
minikube service coordinator --url -n cicd
minikube ip
curl -X POST http://192.168.49.2:32000/build_task \
-H "Content-Type: application/json" \
-d "{\"id\": 1, \"repository\":\"/python/app.py\", \"task\":\"code-review\"}"

curl -X POST http://192.168.49.2:32000/build_task -H "Content-Type: application/json" -d '{"id": 1, "repository": "client", "task": "rust-build"}'
curl -X POST http://192.168.49.2:32000/build_task -H "Content-Type: application/json" -d '{"id": 1, "repository": "client", "task": "python-lint2"}'
kubectl logs -l app=worker -n cicd --tail=50


# testing that traffic is split between workers
kubectl exec -it $(kubectl get pod -l app=coordinator -n cicd -o jsonpath="{.items[0].metadata.name}") -n cicd -- \
bash -c 'for i in {1..10}; do curl -X POST http://worker:5001/execute_task -H "Content-Type: application/json" -d "{\"id\": $i, \"repository\": \"test-repo\", \"branch\": \"main\"}"; sleep 1; done'



# Check that messages were distributed between workers
for pod in $(kubectl get pods -l app=worker -n cicd -o jsonpath="{.items[*].metadata.name}"); do
    echo "📌 Logs from $pod:"
    kubectl logs $pod -n cicd | grep "Received task"
done


for i in {1..1000}; do
    curl -X POST http://localhost:63976/build_task \
        -H "Content-Type: application/json" \
        -d "{\"id\": $i, \"repository\":\"test-repo\", \"branch\":\"main\"}"
done

for i in {1..1000}; do
  curl -X POST http://192.168.49.2:32000/build_task -H "Content-Type: application/json" -d '{"id": 3, "repository": "client", "task": "python-lint2"}'
done
# Go
go mod tidy
WORKER_PORT=5001 sudo go run worker.go

sudo chmod 666 /var/run/docker.sock

curl -X POST http://localhost:5001/execute_task \
     -H "Content-Type: application/json" \
     -d '{
          "id": 3,
          "repo_path": "/home/maso77/repos/distributed_cicd/python_app",
          "type": "python-lint"
        }'

curl -X POST http://localhost:5001/execute_task \
     -H "Content-Type: application/json" \
     -d '{
          "id": 3,
          "repo_path": "/home/maso77/repos/distributed_cicd/python",
          "type": "code-review"
        }' | jq


# Minio
sudo docker run -d -p 9000:9000 -p 9001:9001 \
  --name minio \
  -e "MINIO_ROOT_USER=minioadmin" \
  -e "MINIO_ROOT_PASSWORD=minioadmin" \
  quay.io/minio/minio server /data --console-address ":9001"

mc alias remove myminio && mc alias set myminio http://localhost:9000 minioadmin minioadmin
mc alias list
mc ls myminio
mc cp go.mod myminio/codebucket/

# Verify
kubectl exec -it <worker-pod-name> -- ls /shared
kubectl exec -it <worker-pod-name> -- ls /client


# Grafana


# Tests
kubectl get pods -n cicd
kubectl delete pod coordinator-7d678b4b44-4sbrz -n cicd
kubectl delete pod worker-5f884bf6-x22wd -n cicd
kubectl get pods -n cicd -w

for i in {1..1000}; do
  curl -X POST http://192.168.49.2:32000/build_task -H "Content-Type: application/json" -d '{"id": 3, "repository": "client", "task": "python-lint2"}'
done

curl -X POST http://192.168.49.2:32000/build_task \
-H "Content-Type: application/json" \
-d "{\"id\": 1, \"repository\":\"/python/app.py\", \"task\":\"code-review\"}" | jq
