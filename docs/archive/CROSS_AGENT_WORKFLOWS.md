# Cross-Agent Workflow Patterns for ProntoDB

## Overview

This document outlines proven workflow patterns for coordinating multiple AI agents using ProntoDB as the central state management system. These patterns leverage ProntoDB's multi-user isolation, cursor-based context switching, and TTL cache capabilities.

---

## 🏗️ **Core Workflow Architecture**

### **Agent Hierarchy Pattern**
```
Orchestrator Agent (--user orchestrator --cursor main)
    ├── Worker Agents (--user worker1/2/3 --cursor tasks)
    ├── Monitor Agent (--user monitor --cursor logs) 
    └── Resource Manager (--user resources --cursor allocation)
```

### **Database Context Strategy**
```
main.db        # Orchestration and global state
tasks.db       # Work distribution and task tracking
logs.db        # Monitoring and audit trail
resources.db   # Resource allocation and limits
dev.db         # Development/testing isolation
```

---

## 📋 **Fundamental Workflow Patterns**

### **1. Task Distribution Pattern**

**Orchestrator Setup:**
```bash
# Initialize workflow
prontodb --user orchestrator --cursor main set workflow.status "initializing"
prontodb --user orchestrator --cursor main set workflow.total_tasks 100
prontodb --user orchestrator --cursor main set workflow.completed 0

# Create task queue
for i in {1..100}; do
  prontodb --user orchestrator --cursor tasks set queue.task_$i "pending"
done
```

**Worker Registration:**
```bash
# Worker registration and heartbeat
prontodb --user worker1 --cursor main set agents.worker1.status "active" 
prontodb --user worker1 --cursor main set agents.worker1.last_seen "$(date)"
prontodb --user worker1 --cursor main set agents.worker1.capabilities "processing,analysis"
```

**Task Claiming:**
```bash
# Atomic task claiming pattern
prontodb --user worker1 --cursor tasks get queue.task_1
if [ $? -eq 0 ]; then
  prontodb --user worker1 --cursor tasks set claimed.task_1 "worker1:$(date)"
  prontodb --user worker1 --cursor tasks del queue.task_1
fi
```

### **2. State Synchronization Pattern**

**Shared State Updates:**
```bash
# Update progress with atomic increments
CURRENT=$(prontodb --user orchestrator --cursor main get workflow.completed)
NEW_COUNT=$((CURRENT + 1))
prontodb --user orchestrator --cursor main set workflow.completed $NEW_COUNT

# Progress reporting
PROGRESS=$((NEW_COUNT * 100 / 100))  # Assuming 100 total tasks
prontodb --user orchestrator --cursor main set workflow.progress_percent $PROGRESS
```

**Status Broadcasting:**
```bash
# Broadcast critical status changes
prontodb --user orchestrator --cursor main set broadcast.message "System maintenance in 5 minutes"
prontodb --user orchestrator --cursor main set broadcast.timestamp "$(date)"
prontodb --user orchestrator --cursor main set broadcast.priority "high"
```

### **3. Resource Coordination Pattern**

**Resource Pool Management:**
```bash
# Initialize resource pools
prontodb --user resources --cursor allocation set pools.cpu_cores 8
prontodb --user resources --cursor allocation set pools.memory_gb 32
prontodb --user resources --cursor allocation set pools.gpu_units 2

# Resource allocation
prontodb --user worker1 --cursor allocation set requests.worker1.cpu_cores 2
prontodb --user worker1 --cursor allocation set requests.worker1.memory_gb 4
```

**Resource Conflict Resolution:**
```bash
# Check availability before allocation
AVAILABLE_CPU=$(prontodb --user resources --cursor allocation get pools.cpu_cores)
REQUESTED_CPU=$(prontodb --user worker1 --cursor allocation get requests.worker1.cpu_cores)

if [ $AVAILABLE_CPU -ge $REQUESTED_CPU ]; then
  NEW_AVAILABLE=$((AVAILABLE_CPU - REQUESTED_CPU))
  prontodb --user resources --cursor allocation set pools.cpu_cores $NEW_AVAILABLE
  prontodb --user resources --cursor allocation set allocated.worker1.cpu_cores $REQUESTED_CPU
fi
```

---

## 🔄 **Advanced Workflow Patterns**

### **4. Pipeline Processing Pattern**

**Multi-Stage Pipeline:**
```bash
# Stage definitions
prontodb --user orchestrator --cursor main set pipeline.stages "ingest,process,analyze,output"
prontodb --user orchestrator --cursor main set pipeline.current_stage "ingest"

# Stage-specific queues
prontodb --user ingest_worker --cursor tasks set stage.ingest.item_$ID "ready"
prontodb --user process_worker --cursor tasks get stage.ingest.item_$ID
prontodb --user process_worker --cursor tasks set stage.process.item_$ID "processing"
```

**Progress Tracking:**
```bash
# Track items through pipeline
prontodb --user monitor --cursor logs set pipeline.item_$ID.stage "process"
prontodb --user monitor --cursor logs set pipeline.item_$ID.started "$(date)"
prontodb --user monitor --cursor logs set pipeline.item_$ID.worker "process_worker"
```

### **5. Fault Tolerance Pattern**

**Heartbeat System:**
```bash
# Worker heartbeat (run every 30 seconds)
prontodb --user worker1 --cursor main set heartbeat.worker1 "$(date +%s)"

# Orchestrator health check
LAST_HEARTBEAT=$(prontodb --user orchestrator --cursor main get heartbeat.worker1)
CURRENT_TIME=$(date +%s)
if [ $((CURRENT_TIME - LAST_HEARTBEAT)) -gt 90 ]; then
  echo "Worker1 appears offline"
  prontodb --user orchestrator --cursor main set agents.worker1.status "offline"
fi
```

**Task Recovery:**
```bash
# Detect orphaned tasks and reassign
prontodb --user orchestrator --cursor tasks scan claimed | while IFS='=' read -r key value; do
  WORKER=$(echo $value | cut -d':' -f1)
  CLAIM_TIME=$(echo $value | cut -d':' -f2)
  
  WORKER_STATUS=$(prontodb --user orchestrator --cursor main get agents.$WORKER.status)
  if [ "$WORKER_STATUS" = "offline" ]; then
    TASK_ID=$(echo $key | sed 's/claimed\.//')
    prontodb --user orchestrator --cursor tasks set queue.$TASK_ID "pending"
    prontodb --user orchestrator --cursor tasks del $key
  fi
done
```

### **6. Dynamic Scaling Pattern**

**Load-Based Scaling:**
```bash
# Monitor queue depth
QUEUE_SIZE=$(prontodb --user orchestrator --cursor tasks keys queue | wc -l)
ACTIVE_WORKERS=$(prontodb --user orchestrator --cursor main keys agents | grep status | wc -l)

if [ $QUEUE_SIZE -gt $((ACTIVE_WORKERS * 10)) ]; then
  prontodb --user orchestrator --cursor main set scaling.action "scale_up"
  prontodb --user orchestrator --cursor main set scaling.target_workers $((ACTIVE_WORKERS + 2))
fi
```

---

## 🕐 **TTL-Based Patterns**

### **7. Session Management Pattern**

**Temporary Sessions:**
```bash
# Create session cache (1-hour expiry)
prontodb create-cache sessions.active 3600

# Agent session tracking
prontodb set sessions.active.worker1 "active:task_processing:$(date)"
prontodb set sessions.active.monitor "active:health_check:$(date)"

# Sessions auto-expire after 1 hour
```

**Cache-Based Coordination:**
```bash
# Temporary coordination cache (5-minute expiry)
prontodb create-cache coordination.temp 300

# Quick status updates
prontodb set coordination.temp.pause_requests "worker1,worker2"
prontodb set coordination.temp.priority_shift "high_memory_tasks"
```

### **8. Rate Limiting Pattern**

**API Rate Limiting:**
```bash
# Rate limit cache (1-minute windows)
prontodb create-cache rate_limits.api 60

# Track API calls per worker
CURRENT_CALLS=$(prontodb get rate_limits.api.worker1_calls 2>/dev/null || echo "0")
if [ $CURRENT_CALLS -lt 100 ]; then
  NEW_CALLS=$((CURRENT_CALLS + 1))
  prontodb set rate_limits.api.worker1_calls $NEW_CALLS
  # Proceed with API call
else
  echo "Rate limit exceeded, waiting..."
  sleep 1
fi
```

---

## 🔍 **Monitoring and Observability Patterns**

### **9. Metrics Collection Pattern**

**Performance Metrics:**
```bash
# Worker performance tracking
prontodb --user monitor --cursor logs set metrics.worker1.tasks_completed $TASK_COUNT
prontodb --user monitor --cursor logs set metrics.worker1.avg_task_time $AVG_TIME
prontodb --user monitor --cursor logs set metrics.worker1.error_rate $ERROR_RATE

# System-wide metrics
prontodb --user monitor --cursor logs set metrics.system.total_throughput $THROUGHPUT
prontodb --user monitor --cursor logs set metrics.system.active_agents $AGENT_COUNT
```

**Health Dashboards:**
```bash
# Generate status report
echo "=== System Status ==="
prontodb --user orchestrator --cursor main get workflow.status
echo "Progress: $(prontodb --user orchestrator --cursor main get workflow.progress_percent)%"
echo "Active Workers: $(prontodb --user orchestrator --cursor main keys agents | grep status | wc -l)"
echo "Queue Depth: $(prontodb --user orchestrator --cursor tasks keys queue | wc -l)"
```

### **10. Audit Trail Pattern**

**Action Logging:**
```bash
# Log all significant actions
LOG_ENTRY="$(date):worker1:task_completed:$TASK_ID:success"
prontodb --user monitor --cursor logs set audit.$(date +%s) "$LOG_ENTRY"

# Decision point logging
DECISION="$(date):orchestrator:scale_decision:queue_depth=$QUEUE_SIZE:action=scale_up"
prontodb --user monitor --cursor logs set decisions.$(date +%s) "$DECISION"
```

---

## 🚀 **Deployment Patterns**

### **11. Multi-Environment Pattern**

**Environment Isolation:**
```bash
# Setup environment cursors
prontodb cursor set dev ./dev.db
prontodb cursor set staging ./staging.db  
prontodb cursor set prod ./prod.db

# Environment-specific deployments
prontodb --cursor dev set config.debug_mode "true"
prontodb --cursor staging set config.debug_mode "false"  
prontodb --cursor prod set config.debug_mode "false"

# Agent deployment per environment
prontodb --user dev_orchestrator --cursor dev set workflow.status "testing"
prontodb --user prod_orchestrator --cursor prod set workflow.status "production"
```

### **12. Backup and Recovery Pattern**

**Automated Backups:**
```bash
# Backup critical databases before major operations
prontodb backup --output ./backups/pre-deployment-$(date +%Y%m%d)

# Backup rotation
find ./backups -name "*.db" -mtime +7 -delete  # Keep 7 days
```

**Recovery Procedures:**
```bash
# Emergency recovery
prontodb backup --restore ./backups/last-known-good.db

# Selective cursor restoration
prontodb cursor delete corrupted_db
prontodb cursor set main ./backups/main-backup.db
```

---

## 🎯 **Best Practices**

### **Agent Coordination Guidelines**

1. **Use Consistent Naming:** Always use predictable patterns for keys and users
2. **Implement Heartbeats:** Regular health checks prevent orphaned tasks
3. **Atomic Operations:** Use get-check-set patterns for race condition avoidance
4. **Graceful Degradation:** Design workflows to handle agent failures
5. **TTL for Temporary Data:** Use TTL caches for sessions and temporary coordination

### **Performance Optimization**

1. **Batch Operations:** Group related operations to reduce database calls
2. **Efficient Querying:** Use specific key prefixes for faster scanning
3. **Resource Management:** Track and limit resource consumption per agent
4. **Cache Strategy:** Use TTL caches for frequently accessed temporary data

### **Security Considerations**

1. **User Isolation:** Each agent should have its own user context
2. **Database Separation:** Use different cursors/databases for different security domains
3. **Audit Trails:** Log all significant decisions and actions
4. **Access Patterns:** Restrict agents to only necessary data access

---

## 📚 **Example Implementation Scripts**

See the `/examples` directory for complete, runnable examples:

- `orchestrator_agent.sh` - Master coordination agent
- `worker_agent.sh` - Generic worker agent template  
- `monitor_agent.sh` - System monitoring and health checks
- `resource_manager.sh` - Resource allocation and limits
- `pipeline_processor.sh` - Multi-stage pipeline implementation

---

## 🔄 **Integration with External Systems**

### **CI/CD Integration**
```bash
# Deployment coordination
prontodb --user ci_agent --cursor deployment set status "deploying"
prontodb --user ci_agent --cursor deployment set version "$BUILD_VERSION"
prontodb --user ci_agent --cursor deployment set timestamp "$(date)"
```

### **Monitoring System Integration**
```bash
# Export metrics to external monitoring
METRICS=$(prontodb --user monitor --cursor logs scan metrics)
curl -X POST "$MONITORING_ENDPOINT" -d "$METRICS"
```

---

**These patterns provide a robust foundation for building scalable, fault-tolerant multi-agent systems using ProntoDB as the coordination backbone.**