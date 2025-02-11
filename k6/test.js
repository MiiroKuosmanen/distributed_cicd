import http from "k6/http";
import { sleep } from "k6";

export let options = {
  stages: [
    { duration: "10s", target: 50 }, // Ramp up to 50 users in 10s
    { duration: "30s", target: 50 }, // Keep 50 users for 30s
    { duration: "10s", target: 0 },  // Ramp down
  ],
};

export default function () {
  let payload = JSON.stringify({
    id: Math.floor(Math.random() * 1000),
    repository: "test-repo",
    branch: "main",
  });

  let params = {
    headers: { "Content-Type": "application/json" },
  };

  http.post("http://coordinator.cicd.svc.cluster.local:55522/build_task", payload, params);
  sleep(1);
}

