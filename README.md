# ECE-1724-RUST-Project
> Team: Tianchen Lan(1006285495)/ Yuhan Cui (1005982661) / Yiwen Zheng (1005986293)
 
## 1. Motivation
Personal finance tools are either web applications that trade privacy for convenience or lightweight CLI utilities that remain at the level of logging. What we want is something in between: a fast, distraction-free terminal experience with strict accounting  and the reliability of a secure, self-hosted backend.

In addition, our team members had previously developed similar projects using Python or Java. Those projects were successful, but we always encountered the same problems: rounding errors in currency, messy null values, and difficult-to-manage synchronization issues when importing large amounts of data. This time, we want to try implementing it using Rust.

The core part of personal finance involves recording income and expenses, categorizing them, supporting multiple accounts (such as savings accounts and credit card accounts), and verifying against bills. This is perfectly aligned with the advantages of Rust, and it doesn't require the use of overly complex frameworks. The powerful type system and precise decimal processing enable us to naturally implement rules that were previously impossible to reliably guarantee. We can incorporate most of the correctness stipulations into the type system and the compiler.

Rust provides us with a brand-new starting point without overcomplicating the concept. Strong typing and precise decimal processing help us avoid common "floating-point errors", while the compiler guides us to adopt a simple and reliable model. With Ratatui, we can maintain a simple and unobstructed interface in the terminal, and a small self-hosted HTTPS backend enables the tool to synchronize data and generate reports across different devices. This privacy-first, terminal-friendly approach is still relatively rare in the Rust ecosystem, and we believe this will be a good attempt.

We also place great emphasis on the use of tools. The interface of Ratatui can maintain the focus and efficiency of the workflow in the terminal, and it can be accessed via SSH and is easy to write scripts. When combined with a small self-hosted HTTPS backend, it brings us most of the advantages that command-line tools lack: multi-device synchronization, import pipelines, and optional budget and reports.

We developed this application also for the purpose of learning, but this learning was based on practical operation. If we can open the application, input the complex split purchase information, compare it with the bank statement, and trust these numbers, then we have achieved the goal that we attempted to reach using Python/Java but failed to fully accomplish before: a modern, terminal-oriented, privacy-respecting financial tracking tool.


2. **Objective and Key Features**  
   The objective of this project is to implement a personal finance tracker using Rust that allows users to efficiently manage their income, expenses, and accounts through a command-line interface. Meanwhile ensuring data persistence and accessibility via a secure HTTPS back-end server. The key features are listed as follows:

**Core Features:**

* Transaction Logging: Users can record both income and expense transactions, with support for multiple categories.  
* Categories Management: Users can create and customize categories to organize their financial activities.  
*  Multiple Account Types: Support for different account types such as checking, savings, and credit cards, enabling a more realistic personal finance model.  
* Complex Transactions: Log a single transaction that slips into multiple categories.

**Enhanced Features:**

* Multi-Currency Support: Users can store data in different currencies, and the system will automatically convert it to the primary currency.  
*  Offline Usage and HTTPS Synchronization: Allows users to continue using the app locally while offline, with secure synchronization to the backend server once online.  
* Budget Management: Set a monthly budget or saving goal, display remaining balance, and provide overspending reminders.  

  The project is designed to be completed within the course timeframe and has multiple tasks that can be divided between three team members:   
* **Front-end CLI development:**    
  *  Build an interactive text-based user interface using *Ratatui*.  
  * Implement transaction input and category selection UI.  
  *  Display account balances, budgets, and financial reports in a clear and intuitive way.  
* **Back-end API and server implementation:**  
  * Develop the backend using *Axum*.  
  * Design and implement *HTTPS* endpoints for transactions, accounts, categories, and reports.  
  * Implement offline synchronization logic, ensuring data consistency between local client and backend.  
* **Database schema design:**  
  * Design the database scheme using *SQLx* for transactions, accounts, categories, and budgets.  
  * Implement complex transaction handling.  
  * Support budget tracking, storing recurring transactions.  
  * Build data aggregation queries for financial reports and savings goals.