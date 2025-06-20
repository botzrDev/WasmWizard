# Git Workflow Visual Diagrams

**WasmWiz Project Workflow Visualization**  
**Created:** June 20, 2025

## Complete Workflow Overview

```mermaid
graph TD
    A[Developer Local Work] --> B[Create Feature Branch]
    B --> C[Code Changes]
    C --> D[Local Testing]
    D --> E{Tests Pass?}
    E -->|No| C
    E -->|Yes| F[Commit & Push]
    F --> G[Create Pull Request]
    G --> H[CI Pipeline Triggered]
    
    H --> I[Run Tests]
    H --> J[Security Scan]  
    H --> K[Quality Check]
    
    I --> L{All Checks Pass?}
    J --> L
    K --> L
    
    L -->|No| M[Notify Developer]
    M --> N[Fix Issues]
    N --> C
    
    L -->|Yes| O[Code Review]
    O --> P{Approved?}
    P -->|No| N
    P -->|Yes| Q[Merge to Master]
    
    Q --> R[Full CI/CD Pipeline]
    R --> S[Docker Build]
    S --> T[Deploy to Staging]
    T --> U[Staging Tests]
    U --> V{Staging OK?}
    
    V -->|No| W[Rollback Staging]
    V -->|Yes| X[Await Approval]
    
    X --> Y{Production Approved?}
    Y -->|No| Z[Stay in Staging]
    Y -->|Yes| AA[Deploy to Production]
    
    AA --> BB[Production Health Check]
    BB --> CC{Production OK?}
    CC -->|No| DD[Auto/Manual Rollback]
    CC -->|Yes| EE[Deployment Complete]
```

## Branch Flow Diagram

```mermaid
gitgraph
    commit id: "Initial"
    branch feature/auth
    checkout feature/auth
    commit id: "Add auth middleware"
    commit id: "Add tests"
    checkout main
    merge feature/auth
    commit id: "Deploy v1.1"
    branch hotfix/security
    checkout hotfix/security
    commit id: "Fix security issue"
    checkout main
    merge hotfix/security
    commit id: "Deploy v1.1.1"
    branch feature/api
    checkout feature/api
    commit id: "New API endpoint"
    commit id: "Update docs"
    checkout main
    merge feature/api
    commit id: "Deploy v1.2"
```

## CI/CD Pipeline Detailed Flow

```mermaid
graph LR
    A[Code Push] --> B[GitHub Actions]
    
    subgraph "Parallel Jobs"
        B --> C[Unit Tests]
        B --> D[Integration Tests]
        B --> E[Functional Tests]
        B --> F[Security Scan]
        B --> G[Code Quality]
    end
    
    C --> H{All Jobs Pass?}
    D --> H
    E --> H
    F --> H
    G --> H
    
    H -->|No| I[Fail Build]
    H -->|Yes| J[Docker Build]
    
    J --> K[Image Security Scan]
    K --> L[Push to Registry]
    L --> M[Deploy to Staging]
    
    M --> N[Health Checks]
    N --> O{Staging Healthy?}
    O -->|No| P[Rollback Staging]
    O -->|Yes| Q[Manual Approval Gate]
    
    Q --> R{Approved for Prod?}
    R -->|No| S[Stay in Staging]
    R -->|Yes| T[Deploy to Production]
    
    T --> U[Production Health Checks]
    U --> V{Production Healthy?}
    V -->|No| W[Production Rollback]
    V -->|Yes| X[Success Notification]
```

## Security Scanning Flow

```mermaid
graph TD
    A[Code Changes] --> B[Security Pipeline]
    
    B --> C[Cargo Audit]
    B --> D[Cargo Deny]
    B --> E[Docker Image Scan]
    B --> F[Dependency License Check]
    
    C --> G{Vulnerabilities Found?}
    D --> H{Policy Violations?}
    E --> I{Image Vulnerabilities?}
    F --> J{License Issues?}
    
    G -->|Yes| K[Block Pipeline]
    H -->|Yes| K
    I -->|Yes| K
    J -->|Yes| K
    
    G -->|No| L[Security Pass]
    H -->|No| L
    I -->|No| L
    J -->|No| L
    
    K --> M[Developer Notification]
    M --> N[Fix Security Issues]
    N --> A
    
    L --> O[Continue Pipeline]
```

## Deployment Strategy Visualization

```mermaid
graph TD
    A[Master Branch] --> B[CI Pipeline Success]
    B --> C[Staging Environment]
    
    subgraph "Staging Environment"
        C --> D[2 Replicas]
        D --> E[Health Checks]
        E --> F[Integration Tests]
        F --> G[Performance Tests]
    end
    
    G --> H{Staging Validation}
    H -->|Fail| I[Fix Issues]
    I --> A
    H -->|Pass| J[Manual Approval]
    
    J --> K{Approved?}
    K -->|No| L[Stay in Staging]
    K -->|Yes| M[Production Environment]
    
    subgraph "Production Environment"
        M --> N[3 Replicas]
        N --> O[Rolling Update]
        O --> P[Health Monitoring]
        P --> Q[Traffic Validation]
    end
    
    Q --> R{Production Healthy?}
    R -->|No| S[Automatic Rollback]
    R -->|Yes| T[Deployment Success]
    
    S --> U[Previous Version]
    U --> V[Incident Response]
```

## Rollback Decision Tree

```mermaid
graph TD
    A[Issue Detected] --> B{Severity Level?}
    
    B -->|Critical| C[Immediate Rollback]
    B -->|High| D[Quick Assessment]
    B -->|Medium| E[Detailed Analysis]
    
    C --> F[Execute Rollback Workflow]
    
    D --> G{Can Fix in <15min?}
    G -->|Yes| H[Apply Hotfix]
    G -->|No| F
    
    E --> I{Can Fix in <1hr?}
    I -->|Yes| J[Schedule Fix]
    I -->|No| K[Plan Rollback]
    K --> F
    
    F --> L[Select Rollback Target]
    L --> M[Execute Rollback]
    M --> N[Verify Rollback]
    N --> O[Monitor System]
    
    H --> P[Test Fix]
    P --> Q{Fix Works?}
    Q -->|Yes| R[Deploy Fix]
    Q -->|No| F
    
    J --> S[Implement Fix]
    S --> T[Test in Staging]
    T --> U[Deploy to Production]
```

## Quality Gate Process

```mermaid
graph LR
    A[Code Change] --> B[Quality Gates]
    
    subgraph "Automated Checks"
        B --> C[Format Check]
        B --> D[Lint Check]
        B --> E[Test Coverage]
        B --> F[Security Scan]
    end
    
    C --> G{Formatting OK?}
    D --> H{Lints Pass?}
    E --> I{Coverage >80%?}
    F --> J{Security Clear?}
    
    G -->|No| K[cargo fmt]
    H -->|No| L[Fix Lints]
    I -->|No| M[Add Tests]
    J -->|No| N[Fix Security]
    
    K --> O[Re-run Checks]
    L --> O
    M --> O
    N --> O
    
    G -->|Yes| P[Quality Pass]
    H -->|Yes| P
    I -->|Yes| P
    J -->|Yes| P
    
    P --> Q[Proceed to Review]
    O --> B
```

## Monitoring and Alerting Flow

```mermaid
graph TD
    A[Application Running] --> B[Metrics Collection]
    
    B --> C[Health Checks]
    B --> D[Performance Metrics]
    B --> E[Error Rates]
    B --> F[Resource Usage]
    
    C --> G{Health Status}
    D --> H{Performance OK?}
    E --> I{Error Rate Normal?}
    F --> J{Resources OK?}
    
    G -->|Unhealthy| K[Alert Team]
    H -->|Degraded| K
    I -->|High Errors| K
    J -->|Resource Issues| K
    
    K --> L[Incident Response]
    L --> M{Rollback Needed?}
    M -->|Yes| N[Execute Rollback]
    M -->|No| O[Fix in Place]
    
    G -->|Healthy| P[Continue Monitoring]
    H -->|Good| P
    I -->|Normal| P
    J -->|Normal| P
```

## Development Lifecycle

```mermaid
graph TD
    A[Issue Created] --> B[Assign Developer]
    B --> C[Create Feature Branch]
    C --> D[Development Work]
    
    D --> E[Local Testing]
    E --> F{Tests Pass?}
    F -->|No| D
    F -->|Yes| G[Commit Changes]
    
    G --> H[Push to GitHub]
    H --> I[Create Pull Request]
    I --> J[CI Pipeline Runs]
    
    J --> K{CI Passes?}
    K -->|No| L[Fix Issues]
    L --> D
    K -->|Yes| M[Code Review]
    
    M --> N{Review Approved?}
    N -->|No| O[Address Feedback]
    O --> D
    N -->|Yes| P[Merge to Master]
    
    P --> Q[Deploy to Staging]
    Q --> R[Staging Validation]
    R --> S[Production Deployment]
    S --> T[Issue Closed]
```

---

## Usage Instructions

These diagrams illustrate the complete workflow for the WasmWiz project. They can be:

1. **Embedded in Documentation**: Copy the mermaid code into markdown files
2. **Rendered in GitHub**: GitHub automatically renders mermaid diagrams
3. **Used in Presentations**: Export as images for team presentations
4. **Training Materials**: Help new team members understand the process

## Diagram Tools

- **Mermaid**: Used for all diagrams above
- **GitHub**: Automatically renders mermaid in markdown
- **VS Code**: Mermaid preview extension available
- **Online Editor**: https://mermaid.live/ for editing

---

**Maintenance**: Update these diagrams when workflow changes are made.  
**Location**: `DOCS/GIT_WORKFLOW_DIAGRAMS.md`
