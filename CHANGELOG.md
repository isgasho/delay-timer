# Version 0.3.0 

- Stable to stable-rustc compilation, with repair optimization.

1.Compilable at stable by conditional compilation.

2.Balancing performance and user experience (ajust CandyCronStr inner-type and add free-api for TaskBuilder).

3.Support custom setting of time zone.

4.Fix the clock too fast issue.

5.Use `next_second_hand` to solve a schedule problem.

- 
# Version 0.2.0

- Add `tokio-support` and `status-report`  features, support for tokio ecology, internal logic optimization, generate tasks faster, add syntactic sugar to cron-expressions, etc.

1.Enriched a large number of documents, more easy to use.

2.tokio-Runtime is supported.

3.Custom syntactic sugar for Cron expressions is supported, and the API is more friendly.

4.Optimize the internal logic, more secure execution.

5.task supports new features, you can set the maximum number of parallelism, and the task can automatically recycle the handle after completion.

6.Support status reporting, you can get the internal time by DelayTImer, and you can use the Cancel running task API now.

7.Generate more powerful macros for asynchronous task Body, more details you can find in the documentation and examples.

# Version 0.1.0

- delay-timer is a task manager based on a time wheel algorithm, which makes it easy to manage timed tasks, or to periodically execute arbitrary tasks such as closures.

The underlying runtime is currently based on smol, so upper level applications that want to extend asynchronous functionality need to use libraries that are compatible with smol.

Since the library currently includes features such as #[bench], it needs to be developed in a nightly version.
