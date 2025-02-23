#!/bin/bash

# Define Docker Hub username
DOCKER_USER="maso778"

echo "🚀 Building and pushing Worker image..."
#docker build -t $DOCKER_USER/worker:latest worker/
#docker push $DOCKER_USER/worker:latest

docker build -t $DOCKER_USER/worker:latest worker_go/
docker push $DOCKER_USER/worker:latest

echo "🚀 Building and pushing Coordinator image..."
docker build -t $DOCKER_USER/coordinator:latest coordinator/
docker push $DOCKER_USER/coordinator:latest

echo "🔄 Restarting Worker deployment in Kubernetes..."
kubectl rollout restart deployment/worker -n cicd

echo "🔄 Restarting Coordinator deployment in Kubernetes..."
kubectl rollout restart deployment/coordinator -n cicd

echo "✅ Deployment complete! Checking pod status..."
kubectl get pods -n cicd
