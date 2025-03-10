from locust import HttpUser, task, between
import json

class MyUser(HttpUser):
    wait_time = between(0.1, 0.3)  # Adjust wait time between requests if needed

    @task
    def build_task(self):
        payload = {
            "id": 1,
            "repository": "test-repo",
            "task": "python-lint2"
        }
        self.client.post(
            "/build_task",
            data=json.dumps(payload),
            headers={"Content-Type": "application/json"}
        )
    # Optionally, if you want to simulate high concurrency, you can adjust the user count and spawn rate in the locust command.
