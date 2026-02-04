# Architecting Agentic AI Systems for Autonomous Database Management

## A Deep Dive into PostgreSQL Integration with Rust

---

The paradigm shift in artificial intelligence from passive generative models to active, goal-oriented systems marks the emergence of the agentic era. Unlike standard Large Language Models (LLMs) that function as reactive completion engines, an AI agent is an autonomous software entity designed to reason, plan, and execute actions to achieve specific objectives without continuous human intervention.[^1][^2] These systems represent a transition where the LLM serves not merely as a repository of knowledge but as the central reasoning engine—the "brain"—of a broader architectural framework that integrates memory, planning, and external tool use.[^1][^3] In the specialized domain of database engineering, an AI agent functions as an autonomous intermediary between natural language intent and structured data operations, capable of performing complex schema analysis, executing data manipulation language (DML), and managing database migrations within a safe, governed environment.[^4][^5]

---

## 1. The Fundamental Anatomy of AI Agents

The construction of an AI agent is predicated on four primary pillars: the reasoning core, planning capabilities, memory systems, and tool integration.[^1][^6] The reasoning core is almost universally a foundation model, such as GPT-4, Claude, or a high-performance open-source model like LLaMA 3. This core processes unstructured input and determines the sequential steps required to reach a defined goal.[^2][^3] However, the reasoning core remains isolated without supporting infrastructure. Planning allows the agent to decompose a vague query into manageable sub-tasks, while memory provides the persistence needed to track progress and learn from previous interactions.[^1][^2] Tool use, often referred to as function calling, is the mechanism through which the agent interacts with the physical or digital world, such as executing a SQL query on a PostgreSQL instance.[^2][^7]

---

## 2. Cognitive Architecture and Planning Mechanisms

The planning module is essential for overcoming the limitations of single-pass model generation. Complex tasks, such as generating a multi-table join based on a high-level business question, require the agent to think ahead and reflect on its own strategies.[^3][^8] Advanced agents utilize several planning patterns:

| Planning Pattern | Mechanism | Use Case in Database Agents |
|------------------|-----------|----------------------------|
| Chain-of-Thought (CoT) | Sequential reasoning steps before output | Decomposing a natural language question into logical filter conditions[^9][^10] |
| ReAct (Reason + Act) | Interleaving reasoning traces and actions | Executing a query, observing an error, and correcting the syntax[^7][^11] |
| Reflection | Internal evaluation of proposed plans | Critiquing a generated SQL query for potential performance bottlenecks[^3][^5] |
| Sub-task Decomposition | Breaking goals into smaller, independent parts | Separating schema retrieval from actual query generation[^1][^10] |

These patterns ensure that the agent does not merely guess at an answer but systematically approaches the problem, frequently pausing to reassess its path based on new information.[^8][^12]

---

## 3. Memory Structures for Contextual Persistence

Memory in an agentic system is categorized by its duration and function. Short-term memory typically resides within the model's context window, tracking the immediate dialogue and task status.[^1][^2] Long-term memory, however, requires external storage, often implemented via vector databases or structured stores.[^1][^6] Episodic memory retains the logs of past actions and observations, allowing the agent to avoid repeating failed strategies.[^2][^6] Semantic memory stores broader facts, such as the organization's business logic or database naming conventions.[^4][^6] In a PostgreSQL agent, memory might store previous successful queries to serve as few-shot examples for future requests, thereby increasing accuracy and reducing the likelihood of hallucination.[^4][^13]

---

## 4. Inner Workings: The Reasoning and Execution Loop

The operational life of an AI agent is a continuous loop of reasoning and acting. This loop begins when the agent receives a goal, such as "Generate a report on the top ten customers by revenue in the EMEA region for Q3".[^2][^6] The agent does not generate SQL immediately; instead, it enters an initialization phase where it identifies the necessary data points and tools.[^2][^12]

### 4.1 The ReAct Pattern in Technical Detail

The ReAct framework is the standard for modern agentic loops, providing a structured way for models to "think" before they "act".[^7][^11] The cycle consists of three repeating stages:

1. **Thought**: The model generates a natural language reasoning trace. It might realize it lacks the schema for the orders table and decides to call a schema retrieval tool.[^5][^7]
2. **Action**: The agent issues a structured call to a registered tool. This could be a function like `get_schema(table_name="orders")`.[^7][^14]
3. **Observation**: The system executes the tool and feeds the result back to the model. The model "sees" the columns and types for the table and uses this information to inform its next "Thought".[^7][^11]

This iterative process is mathematically represented by the agent's policy π, which is a conditional distribution over the joint action space of language and environment interactions.[^11] Each step is grounded in the accumulated history of thoughts, actions, and observations, allowing the agent to handle dynamic environments where the state may change unexpectedly.[^9][^11]

### 4.2 Data Flow and Lifecycle of a Request

The flow of data through an agentic system is non-linear and governed by the model's orchestration.[^8] When a user submits a natural language query, it first enters the "Question Understanding" stage, where the intent is parsed.[^4] The agent then retrieves relevant context from its memory or external tools—a process known as schema linking—to identify which tables and columns are required.[^4][^12] Once the schema is retrieved, the agent generates a draft SQL query.[^5][^10] This query is not executed immediately; rather, it passes through a validation layer where another agent or a deterministic parser checks for syntax errors or security violations.[^4][^5] If an error is detected, it flows back into the reasoning loop for correction.[^5][^10] Only after successful validation (and potentially human approval) is the data returned to the user or the DML executed.[^5][^15]

---

## 5. Recommended Architecture for a PostgreSQL AI Agent

Building a production-ready PostgreSQL agent requires moving away from monolithic designs where a single model call is expected to handle everything. A "Modular Specialist" architecture is recommended, where different components are responsible for specific stages of the pipeline.[^5][^12][^16]

### 5.1 The Multi-Agent Orchestrator Model

A multi-agent framework decomposes the problem into specialized roles, each with its own prompt and toolset.[^5][^16][^17]

- **The Orchestrator**: The central manager that coordinates the workflow, deciding which specialist to invoke next based on the current state.[^16][^17][^18]
- **The Schema Extractor**: A tool-specialist that queries the PostgreSQL `information_schema` or `pg_catalog` to provide the LLM with the exact DDL and table relationships needed for the specific query.[^5][^19]
- **The SQL Generator**: A model optimized for code generation that receives the user's intent and the extracted schema to produce a high-fidelity SQL query.[^5][^12]
- **The Critic/Corrector**: A secondary model that reviews the generated SQL for logical errors, such as incorrect joins or missing filters, and suggests refinements.[^3][^5][^16]
- **The Execution/Validation Agent**: A deterministic component that handles the mechanical steps of validating SQL syntax, checking for dangerous commands, and executing the final query.[^5]

This separation of concerns prevents "Reasoning Drift," where a model becomes overwhelmed by attempting to perform intent mapping, schema retrieval, and code generation in a single context window.[^12]

### 5.2 Context Management and Schema Extraction

One of the primary failure modes in Text-to-SQL systems is "Context Window Saturation".[^5][^12] Large enterprise databases often have thousands of tables, making it impossible to include the entire schema in a single prompt. Effective architecture utilizes dynamic schema retrieval.[^5][^20] Instead of static prompts, the agent is given a tool to search the database metadata.[^5][^21] This tool might use vector search over table descriptions or a hierarchy where the agent first lists all tables, then requests the detailed schema for only the relevant ones.[^20][^21]

| Feature | Monolithic Approach | Modular Specialist Architecture |
|---------|---------------------|--------------------------------|
| Schema Handling | Full DDL in prompt (fails on large DBs) | Dynamic retrieval of relevant tables only[^5][^12] |
| Error Handling | Fails on syntax error | Iterative correction loop with feedback[^5][^10] |
| Logic | One-pass generation | Multi-step planning and critique[^12][^16] |
| Security | Direct execution (risky) | Layered validation and HITL middleware[^5][^15] |

---

## 6. Building the Agent with Rust

Rust has emerged as a premier language for building agentic systems due to its performance, memory safety, and high-concurrency model.[^14][^22] In an agentic workload, where multiple LLM calls may happen in parallel and complex state must be managed across asynchronous tasks, Rust's ownership model and lack of a Global Interpreter Lock (GIL) provide a significant advantage over Python.[^14][^22]

### 6.1 Core Crate Ecosystem

The Rust ecosystem provides several mature crates for building database-centric AI agents:

- **rig**: A modular and ergonomic framework for building LLM-powered applications. It abstracts across multiple providers (OpenAI, Anthropic, Gemini) and provides built-in support for tool calling and RAG workflows.[^14][^23][^24]
- **sqlx**: An asynchronous, compile-time checked SQL toolkit. It is the preferred driver for PostgreSQL interaction, ensuring that SQL queries are validated against the database at compile time when possible.[^25][^26][^27]
- **tokio**: The industry-standard asynchronous runtime, essential for managing the non-blocking I/O required for agentic loops.[^25][^28]
- **serde / serde_json**: Fundamental for serializing and deserializing the structured data exchanged between the LLM (as JSON tool calls) and Rust (as typed structs).[^14][^17][^29]
- **sqlx_migrator / refinery**: Crates designed for managing database migrations programmatically, which is a key requirement for agents that need to evolve the database schema.[^30][^31]

### 6.2 Implementing the Reasoning Engine in Rust

Using the rig crate, developers can define an agent by specifying its model, preamble (system prompt), and available tools.[^23][^24] The agent type in rig handles the complexities of maintaining conversation state and routing tool calls to the correct implementations.[^23][^32]

```rust
use rig::providers::openai;
use rig::completion::Prompt;

// Example initialization of a database agent
let client = openai::Client::from_env();
let agent = client
    .agent(openai::GPT_4O)
    .preamble("You are a PostgreSQL expert. Use the provided tools to query data.")
    .tool(PostgresQueryTool::new(pool)) // Tool implementation
    .build();
```

The "Brain" is initialized with a system prompt that enforces dialect-aware SQL generation and provides strict instructions on using only provided schema elements to prevent hallucination.[^12][^33]

---

## 7. Tool Implementation: The Interface Between LLM and PostgreSQL

Tools in Rust-based agents are implemented by defining structs that fulfill the Tool trait.[^29][^32] This trait requires the developer to specify the tool's name, its input arguments (as a deserializable struct), and its execution logic.[^29][^32]

### 7.1 Executing Queries and DML

A `PostgresQueryTool` allows the agent to execute SELECT statements or DML like INSERT and UPDATE.[^4][^34] The input to the tool is typically a SQL string generated by the LLM. In the implementation, this string is executed via sqlx.[^25][^27]

```rust
struct QueryArgs {
    sql: String,
}

impl Tool for PostgresQueryTool {
    const NAME: &'static str = "execute_query";
    type Args = QueryArgs;
    type Output = String; // Result set formatted as JSON/CSV
    type Error = DatabaseError;

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let results = sqlx::query(&args.sql)
            .fetch_all(&self.pool)
            .await?;
        // Convert rows to JSON for the LLM to process
        Ok(format_as_json(results))
    }
}
```

To prevent SQL injection, the system should encourage the agent to use bound parameters.[^27] However, because the agent itself is generating the SQL structure, the primary defense is to run the agent with a database user that has restricted permissions, following the principle of least privilege.[^4][^15]

### 7.2 Schema Introspection Tools

The agent must be able to "see" the database to query it. A `SchemaDiscoveryTool` can be implemented to return the DDL for specific tables or a list of all available tables in the current schema.[^5][^19] Using the `db-schema` crate or direct queries to `information_schema.columns`, the tool provides the agent with the metadata needed to construct accurate joins and filter conditions.[^19][^20]

---

## 8. Managing Database Migrations with AI Agents

A unique requirement for a comprehensive database agent is the ability to create and execute migrations. This involves two distinct steps: generating the DDL and managing the migration versioning system.[^30][^35]

### 8.1 Programmatic Migrations with sqlx_migrator

The `sqlx_migrator` crate allows developers to define migrations as Rust objects that implement an `Operation` trait, consisting of an `up` function for applying changes and a `down` function for rollbacks.[^30][^36] When the agent decides to create a migration, it can generate a new Rust module or a SQL file that follows the migration naming convention.[^30][^31]

```rust
// Defining a programmatic migration
pub struct AddUserStatus;

#[async_trait::async_trait]
impl Operation<sqlx::Postgres> for AddUserStatus {
    async fn up(&self, conn: &mut sqlx::PgConnection) -> Result<(), Error> {
        sqlx::query("ALTER TABLE users ADD COLUMN status TEXT")
            .execute(conn)
            .await?;
        Ok(())
    }

    async fn down(&self, conn: &mut sqlx::PgConnection) -> Result<(), Error> {
        sqlx::query("ALTER TABLE users DROP COLUMN status")
            .execute(conn)
            .await?;
        Ok(())
    }
}
```

The agent can then invoke a `Migrator` tool that runs `Plan::apply_all()` to execute all pending migrations.[^30][^36] However, because schema changes are irreversible and potentially destructive, this operation must be gated by a human-in-the-loop workflow.[^15][^37]

---

## 9. Human-in-the-Loop (HITL) for Safe Database Operations

The potential for an AI agent to cause significant damage—such as deleting a table or modifying sensitive financial records—necessitates a robust human-in-the-loop mechanism.[^15][^38] HITL middleware ensures that the agent pauses before executing high-stakes actions, awaiting explicit human approval.[^15][^39]

### 9.1 The Pause-and-Resume Workflow

The architecture for HITL in Rust leverages the language's asynchronous state machines. When the agent attempts to call a "protected" tool (like `execute_dml` or `apply_migrations`), the system does not execute it immediately.[^38][^40] Instead:

1. **Interrupt**: The system captures the tool call and its arguments.[^41][^42]
2. **Checkpointing**: The entire state of the agent—the conversation history and the pending action—is saved to a persistent store.[^18][^41]
3. **Notification**: A message is sent to a human administrator (via Slack, email, or a custom UI) containing the proposed SQL and the agent's reasoning.[^15][^39][^42]
4. **Review**: The human reviews the SQL and can approve, reject, or edit the command.[^37][^38][^39]
5. **Resume**: Upon approval, the system retrieves the checkpointed state and resumes the asynchronous task, executing the tool and returning the result to the agent.[^40][^41][^42]

### 9.2 Implementing HITL with CIBA and Async Rust

For high-security environments, standards like CIBA (Client Initiated Backchannel Authentication) can be used to manage these asynchronous authorizations.[^40] In Rust, this is often implemented as a middleware or a "Specialized Agent" that manages the state of pending tasks.[^14][^18] Because Rust's async functions are compiled into state machines, they can naturally be polled iteratively, allowing a process to "wait" for an external interrupt (like a database entry indicating human approval) without blocking a thread.[^43][^44][^45]

---

## 10. Security and Guardrails in Database Agents

The intersection of LLMs and databases creates a unique security profile. Beyond traditional SQL injection, there is the risk of "prompt injection," where a user might trick the agent into bypassing security rules.[^4][^46]

| Security Layer | Implementation Strategy | Purpose |
|----------------|-------------------------|---------|
| Authentication | Multi-factor authentication for human reviewers | Prevents unauthorized approval of DDL/DML[^15][^46] |
| Least Privilege | Restricted database user for the agent | Limits the blast radius of a compromised or errant agent[^4][^15] |
| Validation | Use of sqlglot or AST-based parsers | Catches syntax errors and blacklisted keywords (e.g., DROP) before execution[^5][^47] |
| Data Masking | PII redaction in prompts and results | Protects sensitive user data from being sent to external LLM providers[^46][^48] |
| Audit Logging | Comprehensive logs of all Thoughts, Actions, and Results | Provides a forensic trail for compliance and debugging[^15][^49] |

By implementing these layers, the system ensures that the agent remains a productive tool rather than a security liability.[^33][^46]

---

## 11. Optimizing Accuracy with RAG and Semantic Models

For enterprise PostgreSQL databases with complex schemas, the agent's accuracy is heavily dependent on the quality of the context provided.[^16][^46] "Semantic Models" bridge the gap between human business terms and technical database schema.[^4][^16]

### 11.1 Table-Augmented Generation (TAG)

Accuracy is improved when the agent can see typical values within the columns it is querying.[^4] A "Data Sample Tool" can provide the agent with 3–5 sample rows from the relevant tables.[^4][^21] This helps the agent understand if a status column contains integers (1, 2, 3) or strings ('active', 'pending').[^4][^21] However, care must be taken to ensure these samples do not contain PII.[^21]

### 11.2 Semantic Mapping and Contextual Retrieval

When the agent identifies a sub-task, it should not be limited to raw DDL. Providing business definitions—such as how "Net Revenue" is calculated from various discount and tax columns—ensures the generated SQL matches business logic.[^4][^16] This metadata can be stored in a vector database and retrieved based on the semantic similarity between the user's question and the documented metrics.[^4][^13][^21]

---

## 12. Future Outlook: The Scaling of Autonomous Data Agents

The move toward agentic systems in database management represents a fundamental change in how organizations interact with data. By leveraging Rust's high-performance runtime and frameworks like rig, developers can build systems that not only query data but actively manage the entire database lifecycle.[^14][^22] As open-source models improve in their ability to follow complex reasoning chains, the need for expensive proprietary models will decrease, allowing for fully local, secure agentic deployments.[^49][^50][^51]

The recommended architecture—a modular, multi-agent system with a strong human-in-the-loop governance layer—provides the necessary balance between autonomy and safety.[^5][^15][^38] By treating the LLM as a reasoning engine and the database as an environment to be explored and managed through structured tools, the next generation of AI agents will empower non-technical users to perform complex data engineering tasks with natural language, while ensuring that the underlying data remains secure and consistent.[^4][^16][^46]

The technical path forward involves deepening the integration between Rust's type-safe query systems and the model's reasoning loops. As we move toward 2026, the maturity of these frameworks will likely lead to "Self-Healing Databases," where agents autonomously detect performance regressions, suggest index changes via migrations, and execute them after human review, all while maintaining a continuous dialogue with the engineering team.[^14][^16][^46]

---

## References

[^1]: LLM Agents: The Enterprise Technical Guide (2025 Architecture) - Aisera. https://aisera.com/blog/llm-agents/

[^2]: LLM Agents: The Complete Guide - TrueFoundry. https://www.truefoundry.com/blog/llm-agents

[^3]: LLM agents: The ultimate guide 2025 - SuperAnnotate. https://www.superannotate.com/blog/llm-agents

[^4]: LLM & AI Models for Text-to-SQL: Modern Frameworks and Implementation Strategies - Promethium. https://promethium.ai/guides/llm-ai-models-text-to-sql/

[^5]: The Six Failures of Text-to-SQL (And How to Fix Them with Agents) - Google Cloud. https://medium.com/google-cloud/the-six-failures-of-text-to-sql-and-how-to-fix-them-with-agents-ef5fd2b74b68

[^6]: Inside an LLM Agent: Memory, Tools, and How to Build One with OpenAI + Python - Medium. https://medium.com/@kate.ruksha/inside-an-llm-agent-memory-tools-and-how-to-build-one-with-openai-python-90faae1aee88

[^7]: Implementing ReAct Agentic Pattern From Scratch - Daily Dose of Data Science. https://www.dailydoseofds.com/ai-agents-crash-course-part-10-with-implementation/

[^8]: Choose a design pattern for your agentic AI system - Cloud Architecture Center. https://docs.cloud.google.com/architecture/choose-design-pattern-agentic-ai-system

[^9]: What is a ReAct Agent? - IBM. https://www.ibm.com/think/topics/react-agent

[^10]: SQL-of-Thought: Multi-agentic Text-to-SQL with Guided Error Correction - arXiv. https://www.arxiv.org/pdf/2509.00581

[^11]: ReAct Loop Architecture - Emergent Mind. https://www.emergentmind.com/topics/react-loop-architecture

[^12]: Architecting State-of-the-Art Text-to-SQL Agents for Enterprise Complexity - Towards AI. https://pub.towardsai.net/architecting-state-of-the-art-text-to-sql-agents-for-enterprise-complexity-629c5c5197b8

[^13]: Building a Reliable Text-to-SQL Pipeline: A Step-by-Step Guide - FireBird Technologies. https://medium.com/firebird-technologies/building-a-reliable-text-to-sql-pipeline-a-step-by-step-guide-pt-2-9bf0f28fc278

[^14]: Why I Built My AI Agent in Rust - Refresh Agent. https://refreshagent.com/engineering/building-ai-agents-in-rust

[^15]: Keeping Humans in the Loop: Building Safer 24/7 AI Agents - ByteBridge. https://bytebridge.medium.com/keeping-humans-in-the-loop-building-safer-24-7-ai-agents-44a3366f94c2

[^16]: Agentic Semantic Model Improvement: Elevating Text-to-SQL Performance - Snowflake. https://www.snowflake.com/en/engineering-blog/agentic-semantic-model-text-to-sql/

[^17]: How to build your first AI agent with MCP in Rust - Composio. https://composio.dev/blog/how-to-build-your-first-ai-agent-with-mcp-in-rust

[^18]: Introduction to Microsoft Agent Framework. https://learn.microsoft.com/en-us/agent-framework/overview/agent-framework-overview

[^19]: db-schema - crates.io. https://crates.io/crates/db-schema

[^20]: How to make LLMs understand very large PostgreSQL databases - Reddit. https://www.reddit.com/r/AI_Agents/comments/1p7f6rs/how_to_make_llms_understand_very_large_postgresql/

[^21]: Bridging Natural Language and Databases: Best Practices for LLM-Generated SQL - Medium. https://medium.com/@vi.ha.engr/bridging-natural-language-and-databases-best-practices-for-llm-generated-sql-fcba0449d4e5

[^22]: Building AxonerAI: A Rust Framework for Agentic Systems - Medium. https://medium.com/@mnjkshrm/building-axonerai-a-rust-framework-for-agentic-systems-cea8e8d73ba0

[^23]: rig - Rust - Docs.rs. https://docs.rs/rig-core

[^24]: 0xPlaygrounds/rig: Build modular and scalable LLM Applications in Rust - GitHub. https://github.com/0xPlaygrounds/rig

[^25]: Mastering Rust Database Access with SQLx, Diesel, and Advanced Techniques - Kite Metric. https://kitemetric.com/blogs/mastering-rust-database-access-with-sqlx-diesel-and-advanced-techniques

[^26]: Choosing a Rust Database Crate in 2023: Diesel, SQLx, or Tokio-Postgres? - Rust Trends. https://rust-trends.com/posts/database-crates-diesel-sqlx-tokio-postgress/

[^27]: Raw SQL in Rust with SQLx - Shuttle. https://www.shuttle.dev/blog/2023/10/04/sql-in-rust

[^28]: Build an AI Discord Bot in Rust with Rig - Medium. https://medium.com/@0thTachi/build-an-ai-discord-bot-in-rust-with-rig-a-step-by-step-guide-7410107ff590

[^29]: Build a Flight Search AI Agent with Rig - Rig Docs. https://docs.rig.rs/guides/advanced/flight_assistant

[^30]: sqlx_migrator - crates.io. https://crates.io/crates/sqlx_migrator

[^31]: rust-db/refinery: Powerful SQL migration toolkit for Rust - GitHub. https://github.com/rust-db/refinery

[^32]: Rig Tools. https://docs.rig.rs/docs/concepts/tools

[^33]: From natural language to SQL queries: How we built Generate SQL with AI - Xata. https://xata.io/blog/how-we-built-generate-sql-with-ai

[^34]: Ali1858/llm_agents: Demonstrate use case of multi and single llm agents system - GitHub. https://github.com/Ali1858/llm_agents

[^35]: pgmold - crates.io. https://crates.io/crates/pgmold

[^36]: iamsauravsharma/sqlx_migrator - GitHub. https://github.com/iamsauravsharma/sqlx_migrator

[^37]: Human in the Loop for Agentic AI in Oracle Integration - Oracle. https://docs.oracle.com/en/cloud/paas/application-integration/human-loop/human-loop-agentic-ai-oracle-integration.html

[^38]: Human in the Loop Middleware in Python: Building Safe AI Agents with Approval Workflows - FlowHunt. https://www.flowhunt.io/blog/human-in-the-loop-middleware-python-safe-ai-agents/

[^39]: Human-in-the-Loop in Agentic Workflows: From Definition to Walkthrough Demo and Use Cases - Orkes. https://orkes.io/blog/human-in-the-loop/

[^40]: Secure "Human in the Loop" Interactions for AI Agents - Auth0. https://auth0.com/blog/secure-human-in-the-loop-interactions-for-ai-agents/

[^41]: Beyond input(): Building Production-Ready Human-in-the-Loop AI Agents with LangGraph - Dev.to. https://dev.to/sreeni5018/beyond-input-building-production-ready-human-in-the-loop-ai-with-langgraph-2en9

[^42]: Human-in-the-Loop - Workflow DevKit. https://useworkflow.dev/docs/ai/human-in-the-loop

[^43]: Using a Rust async function as a polled state machine - Jeff McBride. https://jeffmcbride.net/blog/2025/05/16/rust-async-functions-as-state-machines/

[^44]: How Rust does async differently (and why it matters) - The New Stack. https://thenewstack.io/how-rust-does-async-differently-and-why-it-matters/

[^45]: Understanding Async Await in Rust: From State Machines to Assembly Code - EventHelix. https://www.eventhelix.com/rust/rust-to-assembly-async-await

[^46]: LLM text-to-SQL solutions: Top challenges and tips - K2view. https://www.k2view.com/blog/llm-text-to-sql/

[^47]: pgdump2sqlite - crates.io. https://crates.io/crates/pgdump2sqlite

[^48]: Boosting RAG-based intelligent document assistants using entity extraction, SQL querying, and agents with Amazon Bedrock - AWS. https://aws.amazon.com/blogs/machine-learning/boosting-rag-based-intelligent-document-assistants-using-entity-extraction-sql-querying-and-agents-with-amazon-bedrock/

[^49]: Using PostgreSQL as an LLM Prompt Store — Why It Works Surprisingly Well - Medium. https://medium.com/@pranavprakash4777/using-postgresql-as-an-llm-prompt-store-why-it-works-surprisingly-well-61143a10f40c

[^50]: Rust for AI Agents - Rust Users Forum. https://users.rust-lang.org/t/rust-for-ai-agents/136946

[^51]: Talk to Database in Rust: AI Agent Transforms Text into SQL Queries - YouTube. https://www.youtube.com/watch?v=LmobmmIpU4Q
