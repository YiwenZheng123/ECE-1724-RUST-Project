# ECE-1724-RUST-Project
> Team: Tianchen Lan(1006285495)/ Yuhan Cui (1005982661) / Yiwen Zheng (1005986293)
 
## 1. Motivation
Personal finance tools are either web applications that trade privacy for convenience or lightweight CLI utilities that remain at the level of logging. What we want is something in between: a fast, distraction-free terminal experience with strict accounting  and the reliability of a secure, self-hosted backend.

In addition, our team members had previously developed similar projects using Python or Java. Those projects were successful, but we always encountered the same problems: rounding errors in currency, messy null values, and difficult-to-manage synchronization issues when importing large amounts of data. This time, we want to try implementing it using Rust.

The core part of personal finance involves recording income and expenses, categorizing them, supporting multiple accounts (such as savings accounts and credit card accounts), and verifying against bills. This is perfectly aligned with the advantages of Rust, and it doesn't require the use of overly complex frameworks. The powerful type system and precise decimal processing enable us to naturally implement rules that were previously impossible to reliably guarantee. We can incorporate most of the correctness stipulations into the type system and the compiler.

Rust provides us with a brand-new starting point without overcomplicating the concept. Strong typing and precise decimal processing help us avoid common "floating-point errors", while the compiler guides us to adopt a simple and reliable model. With Ratatui, we can maintain a simple and unobstructed interface in the terminal, and a small self-hosted HTTPS backend enables the tool to synchronize data and generate reports across different devices. This privacy-first, terminal-friendly approach is still relatively rare in the Rust ecosystem, and we believe this will be a good attempt.

We also place great emphasis on the use of tools. The interface of Ratatui can maintain the focus and efficiency of the workflow in the terminal, and it can be accessed via SSH and is easy to write scripts. When combined with a small self-hosted HTTPS backend, it brings us most of the advantages that command-line tools lack: multi-device synchronization, import pipelines, and optional budget and reports.

We developed this application also for the purpose of learning, but this learning was based on practical operation. If we can open the application, input the complex split purchase information, compare it with the bank statement, and trust these numbers, then we have achieved the goal that we attempted to reach using Python/Java but failed to fully accomplish before: a modern, terminal-oriented, privacy-respecting financial tracking tool.
