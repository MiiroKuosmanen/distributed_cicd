import requests
import json
import os
WORKER_URL = "http://localhost:5001/execute_task"

# Define test tasks
tasks = [
    {"id": 1, "repo_path": "/home/maso77/repos/distributed_cicd/client", "type": "rust-build"},
    {"id": 2, "repo_path": "/home/maso77/repos/distributed_cicd/client", "type": "rust-test"},
    {"id": 3, "repo_path": "/home/maso77/repos/distributed_cicd/client", "type": "python-lint"},
]

def send_task(task):
    """Send task request to the worker"""
    headers = {"Content-Type": "application/json"}
    response = requests.post(WORKER_URL, headers=headers, data=json.dumps(task))

    if response.status_code == 200:
        print(f"✅ Task {task['id']} success: {response.json()}")
    else:
        print(f"❌ Task {task['id']} failed: {response.json()}")

if __name__ == "__main__":
    print("🚀 Sending tasks to the worker...\n")
    for task in tasks:
        send_task(task)
