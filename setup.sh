#!/bin/bash

# Function to port-forward a service
port_forward() {
  local service="$1"
  local port_mapping="$2"
  local namespace="$3"

  echo "Starting port-forward for $service in namespace $namespace on ports $port_mapping"

  # Using nohup and & to run each port-forward in the background
  nohup kubectl port-forward svc/"$service" "$port_mapping" -n "$namespace" > /dev/null 2>&1 &
}

# Starting port-forward for mongodb
port_forward my-mongo-release-mongodb 27017:27017 mongodb

# Starting port-forward for prometheus
port_forward prometheus 9090:9090 monitoring

# Starting port-forward for grafana
port_forward grafana 3100:3100 monitoring

# Start the Minikube dashboard
echo "Opening Minikube dashboard..."
minikube dashboard &

echo "All port-forwarding processes and Minikube dashboard have been started."
