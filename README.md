# ECE-1724-RUST-Project

> Team:
> * Yiwen Zheng (1005986293) yiwen.zheng@mail.utoronto.ca
> * Yuhan Cui (1005982661) yuhan.cui@mail.utoronto.ca
> * Tianchen Lan(1006285495) bruce.lan@mail.utoronto.ca

## 📖 Table of Contents

* [1. Motivation](#1-motivation)
* [2. Objective and Key Features](#2-objective-and-key-features)
* [3. User’s Guide](#3-users-guide)
    * [3.1 Global Navigation](#31-global-navigation)
    * [3.2 Accounts Tab](#32-accounts-tab-default-view)
    * [3.3 Transactions Tab](#33-transactions-tab)
    * [3.4 Add / Edit Transaction Tab](#34-add--edit-transaction-tab)
    * [3.5 Dashboard Tab](#35-dashboard-tab)
    * [3.6 Add / Edit Saving Goal Tab](#36-add--edit-saving-goal-tab)
* [4. Reproducibility Guide](#4-reproducibility-guide)
* [5. Contributions](#5-contributions)
* [6. Lessons We Learned](#6-lessons-we-learned)
* [7. Videos](#7-videos)


## 1. Motivation

Personal finance tools are either web applications that trade privacy for convenience or lightweight CLI utilities that remain at the level of logging. What we want is something in between: a fast, distraction-free terminal experience with strict accounting  and the reliability of a secure, self-hosted backend.

In addition, our team members had previously developed similar projects using Python or Java. Those projects were successful, but we always encountered the same problems: rounding errors in currency, messy null values, and difficult-to-manage synchronization issues when importing large amounts of data. This time, we want to try implementing it using Rust.

The core part of personal finance involves recording income and expenses, categorizing them and supporting multiple accounts (such as savings accounts and credit card accounts). This is perfectly aligned with the advantages of Rust, and it doesn't require the use of overly complex frameworks. The powerful type system and precise decimal processing enable us to naturally implement rules that were previously impossible to reliably guarantee. We can incorporate most of the correctness stipulations into the type system and the compiler.

Rust provides us with a brand-new starting point without overcomplicating the concept. Strong typing and precise decimal processing help us avoid common "floating-point errors", while the compiler guides us to adopt a simple and reliable model. With Ratatui, we can maintain a simple and unobstructed interface in the terminal, and a small self-hosted backend enables the tool to synchronize data and generate reports across different devices. This privacy-first, terminal-friendly approach is still relatively rare in the Rust ecosystem, and we believe this will be a good attempt.

We also place great emphasis on the use of tools. The interface of Ratatui can maintain the focus and efficiency of the workflow in the terminal, and it can be accessed via SSH and is easy to write scripts. For data persistence, we utilized a local SQLite database, offering a fast, reliable, and self-contained storage solution. When combined with a small self-hosted backend, which focuses on local data management, ensuring that users have direct control over their financial records with robust storage.

We developed this application also for the purpose of learning, but this learning was based on practical operation. If we can open the application, input the complex split purchase information, compare it with the bank statement, and trust these numbers, then we have achieved the goal that we attempted to reach using Python/Java but failed to fully accomplish before: a modern, terminal-oriented, privacy-respecting financial tracking tool.

## 2. Objective and Key Features

The core objective of this project was to implement a robust, privacy-respecting personal finance tracker using **Rust** that allows users to efficiently manage their accounts and transactions through a terminal-based interface.`

**Core Features:**

* **User Interface:** A clear and concise interface, built with Ratatui, is designed for easy modification and operation within the terminal environment.
* **Transaction Logging:** Users can record both income and expense transactions for multiple categories.
* **Balance Calculation:** The system accurately calculates and displays the current balance for each account of transaction updates.
* **Multiple Account Types:** Support for different account types such as checking, savings, and credit cards, enabling a more realistic personal finance model.
* **Saving Goals:** Allow users to set specific savings targets and  display the progress.
* **Financial Reports:** Capability to generate monthly spending reports and aggregate data for valuable financial insights.

The project is designed to be completed within the course timeframe and has multiple tasks that can be divided between three team members:

* **Front-end CLI development:**
  * Build an interactive text-based user interface using *Ratatui*.
  * Implement transaction input and category selection UI.
  * Display account balances, transactions, and financial reports in a clear and intuitive way.
* **Back-end API and server implementation:**
  * Develop the backend using *Axum*.
  * Design and implement *HTTPS* endpoints for transactions, accounts, categories, and reports.
* **Database schema design:**
  * Design the database scheme using *SQLx* for transactions, accounts, categories.
  * Implement complex transaction handling.
  * Build data aggregation queries for financial reports and savings goals.

## 3. User’s Guide

This application is a terminal-based personal finance tracker built with Rust. It features a modal-based TUI (Text User Interface) for managing accounts, tracking transactions, and visualizing financial health. All hot keys should exactly match the one in the Guide (lower case keys).

### **3.1 Global Navigation**

* **Launch**: Run ```cargo run``` in the terminal to start the application.
* **Switch Tabs**: Press `Tab` to cycle through the main views: `Accounts`, `Transactions`, `Add Transaction`, and `Dashboard`.
* **Help**: Press `?` at any time to toggle the keybindings overlay.
* **Quit**: Press `q` to exit the application.

### **3.2 Accounts Tab (Default View)**

This tab lists all financial accounts and their current balances.

* **View Accounts**: Use `↑ / ↓`arrows to highlight an account.
  * **Visuals**: Positive balances are displayed in <span style="color:green;">**Green**</span>; negative (debt) balances are displayed in <span style="color:red;">**Red**</span>.
* **Create Account**:
  * Press `n` to open the **New Account** modal.
  * Press `Tab` to navigate fields (`Name`, `Type`, `Currency`, `Opening Balance`). Note: once you hit `Enter`, Opening Balance cannot be changed from the Accounts page. You can change the amount by editing in the transaction page.
  * **Account Type**: When the ***Type*** field is selected, use `↑ / ↓` arrows to cycle through options (Checking, Savings, Credit, etc.).
  * **Save**: Press `Enter` on the final field to create the account.
* **Edit Account**: Press `e` on a selected account to modify its name or details.
* **Delete Account**: Press `d`. A warning modal will appear; press `y`or `Enter` to confirm deletion.
* **Select Account**: Press `Enter` on a highlighted account to view its transactions.
* **Go to Dashboard:** Press `g` to direct to the Dashboard tab to manage Saving Goals and view Financial Reports.

### **3.3 Transactions Tab**

Displays the ledger for the currently selected account.

* **Navigation**: Use `↑ / ↓` arrows to scroll through the transaction history.
* **Add Transaction**: Press `a` to jump immediately to the ***Add Transaction*** tab with a blank form.
* **Edit Transaction**: Press `e` on a specific row. This navigates to the ***Add Transaction*** tab but pre-fills the form with that transaction's data for modification.
* **Delete Transaction**: Press `d` or `Delete` to remove the selected transaction.
* **Back**: Press `Esc` to return to the Accounts list.

### **3.4 Add / Edit Transaction Tab**

This screen features a **Data Entry Form** (left) and a **Category Selection List** (right).

* **Navigation**: Press `Tab` to cycle focus between fields.
* **Entering Text (Important)**:
  * To modify a text field (`Date`, `Payee`, `Memo`), press `Enter` to toggle ***Edit Mode***.
  * A generic indicator `(>)`will appear. Type your text, then press `Enter` again to confirm the value and unlock navigation.
* **Selecting Categories**:
  * Navigate to the ***Category*** field.
  * Use `↑ / ↓` arrows to select a category from the right-hand list.
  * **Visuals:** Income categories appear in <span style="color:green;">**Green**</span>; Expense categories appear in <span style="color:red;">**Red**</span>.
* **Amount & Type**:
  * Press `t` to manually toggle the transaction type between <span style="color:green;">**Income (+)**</span> and <span style="color:red;">**Expense (-)**</span>.
* **Save**: Press `Ctrl + s` to save the transaction to the database.
* **Cancel**: Press `Esc` to clear the form or return to the previous view.

### **3.5 Dashboard Tab**

Displays a ***Saving Goals List*** (left) and a ***Finance Report Form*** (right).

**Saving Goals List**

* **Navigation**: Use `↑ / ↓` arrows to select a saving goal from the left-hand list.
* **View Saving Goal**: Press `n` to jump immediately to the ***Create New Saving Goal*** tab with a blank form.
* **Edit Transaction**: Press `e` on a specific row. This navigates to the ***Saving Goal*** tab but pre-fills the form with that transaction's data for modification.
* **Delete Transaction**: Press `d` or `Delete` to remove the selected transaction.

**Finance Report Form**

* **Monthly Spending:** Displays your total spending for the current month by category.
* **Sorting Method:** Expenditures are sorted from largest to smallest, making it easy to quickly compare the size of each expenditure.
* **Back**: Press `Esc` to return to the Accounts list.
* **Cancel**: Press `Esc` to clear the form or return to the previous view.

### **3.6 Add / Edit Saving Goal Tab**

This screen features a **Data Entry Form**.

* **Navigation**: Press `Tab` to cycle focus between fields.
* **View Saving Goal:** Use `↑ / ↓` arrows to highlight a saving goal.
* **Visuals:** Each savings goal will display the savings progress based on the current amount and the target amount.
* **Create A Saving Goal:**
  * Press `n` to open the ***New Saving Goal*** modal.
  * Press `Tab` to navigate fields (`Name`, `Target amount`, `Target amount saved`, `Deadline`).
  * Enter your goal with a corresponding highlighted field.
  * Press `Enter` on the final field to save the saving goal.
* **Edit Goal:** Press `e` on a selected goal to modify its target or current money you have saved.
* **Delete Goal:** Press `d`. A warning modal will appear; press `y` or `Enter` to confirm deletion.
* **Cancel:** Press `Esc` to return to the previous view.

## 4. Reproducibility Guide

### **Getting Started: Your Rust Adventure\!**

To ensure the application runs correctly on a fresh environment, please follow the steps below in order. These commands will set up the necessary dependencies, configure the environment variables, initialize the local SQLite database, and launch the application.

### **Prerequisites:**

* Ensure **Rust** and **Cargo** are installed on the system.
* Ensure **SQLite** is installed (typically pre-installed on macOS and most Linux distributions).

***Haven't installed Rust and SQLite? No worries, [here](#install-rust--cargo) are the steps to load up Cargo\!***

### **Step 0: Project Setup**

Clone the Repository:

```Bash
git clone “https://github.com/YiwenZheng123/ECE-1724-RUST-Project.git”
```

Direct to the project folder:

```
cd personal-finance-tracker
```

### **Step 1: Install SQLx Command Line Tool**

The project relies on the `sqlx-cli` tool to manage database creation and migrations. Run the following command to install it with ***SQLite*** support:

```Bash
cargo install sqlx-cli --no-default-features --features sqlite()`
```

### **Step 2: Initialize and Migrate the Database**

Run the following commands to create the database file and apply the SQL migrations (which set up the table schema and default data):

```Bash
cargo sqlx migrate run
```

### **Step 3: Build and Run the Application**

Finally, with everything installed, you can now build and run the application.

```Bash
cargo build
cargo run
```

#### **Problems:**

If you have encountered a database connection problem check the following notes.

* The application uses a `.env` file to define the database connection string. Since this file is excluded from version control for security, it must be created manually in the project root directory. Run the following command:
  
  ` echo "DATABASE_URL=sqlite://finance_tracker.db" > .env`
* Recreate the database:
  
  ```Bash
  rm finance_tracker.db
  cargo sqlx database create
  cargo sqlx migrate run
  ```

If the build is successful, the application will start, and you are ready to go\!


### Install Rust & Cargo

* Rust is installed via rustup, which is the official toolchain installer.
* Run the installation script:
  ```Bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
* When prompted, choose the default installation.
* Activate Environment: Restart your terminal or run this command to load the necessary environment variables:
  ```Bash
  source $HOME/.cargo/env
  ```
* Verify Installation: Check that both the Rust compiler (rustc) and the package manager (cargo) are installed:

```Bash
    rustc --version 	# e.g., rustc 1.90.0
    cargo --version 	# e.g., cargo 1.90.0
```

**Database Setup (SQLite)**

This project uses ***SQLite**** by default, which is an embedded database.

* Install SQLite. Run the installation script:
  * **macOS:**
  ```brew install sqlite```
  * **Ubuntu:**
  `sudo apt update && sudo apt install -y sqlite3 libsqlite3-dev`
  * **Windows:**
      * Download from sqlite3 website: [https://sqlite.org/download.html](https://sqlite.org/download.html)
      * Select [sqlite-tools-win-x64-3510100.zip](https://sqlite.org/2025/sqlite-tools-win-x64-3510100.zip) or choose based on your system.
* Open your System Environment Variables: `Path → Edit → Add` your path of  the directory where contains the `.exe` files. For example:`C:\\sqlite3`
* Click `Save` to all windows. Then reopen a new terminal
* **Verify Installation:**
  ```Bash
  sqlite3 --version
  ```

## 5. Contributions

### Tianchen Lan:

**1\. Independently Developed**

**Frontend & UI (Ratatui)**

* **Dashboard Interface:** Built the entire Dashboard page layout, splitting the view between ***Savings Goals*** and ***Monthly Spending***.
* **Data Visualization:** Implemented the ***Gauge widget*** for goal progress and the ***BarChart widget*** for expense reports (including logic to handle negative values).
* **Interactivity:** Developed the modal system for creating, editing, and deleting goals.
* **Keyboard Navigation:** Implemented specific key handlers (n, e, d, arrows) to make the dashboard fully interactive via keyboard.

**Backend & Logic (Rust \+ SQLx)**

* **Goal Management:** Wrote the Rust structs and API methods to handle Create, Update, and Delete operations for savings goals.
* **State Management:** Fixed data duplication bugs by implementing proper state tracking (editing\_id) during the edit process.
* **Report Queries:** Optimized SQL queries to correctly aggregate monthly transaction data and map it to the frontend charts.

**2\. Collaborated / Integrated (Team Work)**

* **Database Integration:** Connected the Dashboard logic with the team's existing SQLite database schema.
* **App Persistence:** Worked on the application initialization flow to ensure data is saved and loaded correctly on restart (removing the "wipe on start" behavior).
* **System Consistency:** Integrated dashboard shortcuts into the global Help page and Navigation bar to match the team's design style.

### Yiwen Zheng:

**1\. Independently Developed**

**Frontend & UI (Ratatui)**

* **Global Navigation Structure:** Designed the main application loop, handling tab switching (Accounts, Transactions, Add Transaction) and global event listeners (Help overlay, Quit).
* **Account Management Interface:** Built the Accounts list view with conditional formatting (Green for positive, Red for debt) and implemented the custom modal system for creating and editing accounts.
* **Complex Form Logic:** Engineered the "Add/Edit Transaction" interface, specifically the input state machine that toggles between "Navigation Mode" and "Edit Mode" (using Enter) to handle text inputs in a terminal environment.
* **Interactive Category Selection:** Developed the split-pane view for transactions, allowing users to select categories from a list while simultaneously updating the data entry form.

**Backend & Logic (Rust \+ SQLx)**

* **Core CRUD Implementation:** Wrote the Rust structs and API methods to handle Create, Read, Update, and Delete operations for both Accounts and Transactions.
* **Transaction State Management:** Handled the logic for toggling transaction types (Income vs. Expense) and calculating the resulting mathematical updates to account balances.

**2\. Collaborated / Integrated (Team Work)**

* **View Routing:** Connected the Accounts view to the specific Transaction ledger, ensuring that selecting an account **(`Enter`)** correctly filters and loads the associated transaction history from the database.
* **Database Persistence:** Connected the frontend forms to the shared database backend, ensuring that **`Ctrl+s`** triggers the correct SQL insert/update commands.
* **UX Consistency:** Standardized the keyboard shortcut scheme (arrow keys for navigation, `Esc` for back/cancel) to match the navigation patterns used in the Dashboard module.

### Yuhan Cui:

**1\. Independently Developed**

**Database & Migration (SQLX \+ sqlite)**

* Implemented and tested the database schema and corresponding query functionality to ensure correct data insertion and retrieval; promptly updated missing database attributes when issues were identified.
* Implemented automatic database migration to support future schema changes and simplify database maintenance.

**2\. Collaborated / Integrated (Team Work)**

* Improved team version control by adding a `.gitignore` file to prevent unnecessary files from being tracked.
* Updated application initialization logic to correctly handle database resets with predefined categories, and refined tab spacing on the transaction page to improve UI consistency.
* Tested the application end-to-end and collaborated with teammates to identify and address bugs, enhancing overall system stability.
* Verified the feasibility of the Reproducibility procedure.
* Completed the recording of the video slide presentation of project design and outcomes.

## 6. Lessons We Learned

Throughout the development of this personal finance tracker, our team gained significant insights into systems programming, asynchronous architectures, and collaborative software development.

1. **Collaboration** and **Version Control**
   As the project grew, we encountered challenges with Git merge conflicts, particularly when multiple members modified the database schema or the central `state` logic simultaneously. We adopted a workflow that relies on feature branches and frequent communication before merging to main. We also learned that cleaning build artifacts `cargo clean` is sometimes necessary when switching branches to resolve linker errors caused by stale dependencies.
2. **Managing Database Environment**
   We realized that setting up the database environment is just as important as writing the code. We faced issues where the code wouldn't run on a different computer because of a missing `.env` file or uninstalled tools `sqlx-cli`. This taught us the importance of writing clear documentation and ensuring every team member has the same setup.
3. **Compile-Time Security and Migration**
   We encountered a challenge stemming from **SQLx**'s dedication to **compile-time safety**, where the `query!` macro forces a connection to the database during compilation to strictly verify all SQL statements. This strong mechanism, which prevents common runtime errors: a new database lacked the necessary tables, causing the SQLx compile check to fail. This error prevented us from using cargo run and executing the automatic Migration code within our application. To solve this, we had to manually run the cargo sqlx migrate run before the very first compilation. After this manual step created the tables, the code compiled successfully. From then on, the Rust application code can automatically manage all future schema updates during deployment, preserving Rust's strong compile-time guarantee.
4. **The Learning Curve of Rust**
   One of the biggest challenges was getting used to Rust’s strict compiler rules. At the beginning, we spent a lot of time fighting with syntax errors. However, we learned that these strict rules actually helped us prevent bugs and crashes in the long run. Once the code compiled, it usually ran correctly without issues.

Overall, this project was a great opportunity to build a complete application from scratch. We successfully created a functional finance tracker that ensures data privacy by running locally. While Rust was challenging to learn, we now appreciate its performance and safety features. We are proud of the final result and the teamwork it took to get here.

## 7. Videos:

[Video Demo](https://youtu.be/uA57o_QmwnU)

[Video Slide Presentation](https://drive.google.com/file/d/1YzHOg7ViEjKnOYAugsOB2KfSOpgkZFg_/view?usp=sharing)
