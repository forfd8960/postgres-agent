# Project Instructions


## Project Overview

The Postgres Agent project aims to develop an AI-powered agent that can interact with PostgreSQL databases using natural language. The agent will leverage large language models (OpenAI) to understand user queries and perform database operations such as querying, updating, and managing data. The goal is to create a seamless interface that allows users to interact with their databases without needing to write SQL queries manually.

## Key Features

 - Natural Language Querying: Users can ask questions and make requests in plain English, and the agent will translate these into SQL queries.
 - Database Management: The agent can perform various database management tasks, including: Run DML, Generate Migrations, Create Indexes, Analyze Performance, Backup and Restore.
 - The Agent can allow user configure multiple Postgres DBs in config file and switch between them.
 - Interactive Terminal UI: A terminal-based user interface (TUI) will provide an interactive experience for users to communicate with the agent.
 - Async Operations: The agent will utilize asynchronous programming to handle multiple requests efficiently.
 - Contextual Understanding: The agent will maintain context over a session, allowing for more complex interactions and follow-up questions.
 - Security and Permissions: The agent will respect database security protocols and user permissions to ensure safe operations.
 - Extensibility: The architecture will allow for easy integration of additional features and support for other database systems in the future.

## Tech Stack

we will use Rust to implement the Postgres Agent, leveraging the following key libraries and frameworks:

- clap & ratatui: Command-line interface and TUI framework for building interactive terminal applications.
- sqlx: Asynchronous SQL toolkit for Rust, providing support for PostgreSQL.
- tokio: Asynchronous runtime for Rust, enabling concurrent operations.
- serde: Framework for serializing and deserializing data structures in Rust.
- async-openai(https://docs.rs/async-openai/latest/async_openai/): Async client for OpenAI's API, enabling interaction with large language models. allow user config model, baseurl, api-key, timeout in config file.
- anyhow: Error handling library for Rust, providing context-aware error messages.
- tracing: Instrumentation library for Rust, used for logging and diagnostics.
- dotenvy: Library for loading environment variables from a .env file.

## Write Spec

Write a tech spec for Building Postgres Agent based on above project overview, key features and tech stack. 

refer documents: ./specs/0001_agent_research.md, ./specs/0002_agent_research1.md, ./specs/0003_build_agent.md, ./specs/0004_prd_requirement.md

- Use latest version of crates.
- Follow the best practices of Rust programming and SOLID, DRY, KISS principles.
- Detailed Architecture and Design considerations.
- Detailed Data flow and Interaction diagrams.
- Clearly defined Modules and their responsibilities.
- The Core Traits and Data Structures.
- Async and Concurrency model.
- Error Handling.
- Testing and Validation strategies.

## Update LLM Client

update tech spec that for `OpenAiProvider` to use `async-openai` crate as LlmClient.
Use async-openai crate(https://docs.rs/async-openai/latest/async_openai/) as LlmClient in OpenAiProvider.