from locust import HttpUser, task, between
import json

class CIUser(HttpUser):
    wait_time = between(1, 3)  # Simulate users waiting 1-3 seconds before next request

    @task
    def send_build_task(self):
        headers = {"Content-Type": "application/json"}
        payload = json.dumps({
            "id": 1,
            "repository": "test-repo",
            "branch": "main"
        })
        response = self.client.post("/build_task", data=payload, headers=headers)
        print(f"Response: {response.status_code} {response.text}")
