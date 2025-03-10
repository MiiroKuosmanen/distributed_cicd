import http from "k6/http"; // Import the http module
import { sleep } from "k6"; // Import sleep for controlling wait time
import { check } from "k6"; // Import check for validating the responses

// Set the default options (like VUs and duration)
export let options = {
  stages: [
    { duration: "10s", target: 10 }, // Ramp up to 50 VUs in 30 seconds
    { duration: "10s", target: 10 }, // Stay at 50 VUs for 1 minute
    { duration: "10s", target: 0 }, // Ramp down to 0 VUs in 30 seconds
  ],
};

// Define the test logic (the tasks each virtual user will perform)
export default function () {
  let payload = JSON.stringify({
    id: 1,
    repository: "test-repo",
    task: "python-lint2",
  });

  // Make the POST request to the API endpoint
  let response = http.post("http://192.168.49.2:32000/build_task", payload, {
    headers: { "Content-Type": "application/json" },
  });

  // Optionally check the response status
  check(response, {
    "is status 200": (r) => r.status === 200,
  });

  // Add a random sleep time between requests (simulating think time)
  sleep(Math.random() * (0.3 - 0.1) + 0.1); // Sleep between 0.1 and 0.3 seconds
}
