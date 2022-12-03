### K8S job/cronjob solution

Implement a very simple API server in rust with the following routes that can interact with kubernetes.

- POST /jobs (Create a new job that runs on a kubernetes cluster)
- GET /jobs/stats (Returns aggregate job stats. Succeeded vs failed and no of retries)
- POST /jobs/schedule (Schedule a job using cron syntax)

Job Requirement:

- Each job should spawn a docker container with the following command.
- If the job fails, retry 2 times with a small time delay ideally.

Directory structure:

```
.
├── Cargo.lock
├── Cargo.toml
├── README.md
├── data_processor.zip     -> Job
├── dockerfile             -> Containerize
├── makefile               -> CLI utils
├── src                    -> API server
│   └── main.rs
```

1. Monitoring solution:

   - prometheus, grafana

2. Fault tolerance solution:

   - retry, HA solution

3. Scalability solution:

   - [Pod](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/)
   - [Cluster](https://docs.aws.amazon.com/eks/latest/userguide/autoscaling.html)

4. Running 100s of these jobs in parallel:
   - [Parallel](https://kubernetes.io/docs/concepts/workloads/controllers/job/#parallel-jobs)

#### Curl samples:

1. POST /jobs

```bash
$ curl -XPOST localhost:8080/jobs \
-H 'Content-type: application/json' \
-d '{"name":"test-job"}'
```

2. GET /jobs/stats

```bash
$ curl localhost:8080/jobs/stats
```

3. POST /jobs/schedule

```bash
$ curl -XPOST localhost:8080/jobs/schedule \
-H 'Content-type: application/json' \
-d '{"name":"test-cronjob","syntax":"* * * * *"}'
```
