# 09. 数据模型草案

> Version: 0.1  
> Parent: [GESA 企业空间系统](./README.md)

---

## 1. 定位

本文件只记录企业空间系统第一版所需的核心数据表草案。

完整数据库设计应在开发者架构层继续细化。

---

## 2. 实体对象

```sql
entities
- id
- type              -- human / agent
- name
- status
- created_at
- updated_at

human_profiles
- entity_id
- employee_no
- display_name
- can_be_ceo

agent_profiles
- entity_id
- template_id
- owner_entity_id
- cloneable
- can_be_ceo
```

---

## 3. 组织结构

```sql
org_levels
- id
- level_index
- name
- rank

org_directions
- id
- direction_index
- name
- description

org_cells
- id
- level_id
- direction_id
- occupied_department_id
- unique(level_id, direction_id)
```

---

## 4. 部门

```sql
departments
- id
- name
- head_entity_id
- department_space_id
- status

department_cells
- department_id
- cell_id

department_members
- department_id
- entity_id
- role
- status
```

---

## 5. 空间

```sql
spaces
- id
- type              -- personal / cell / department / direction / level / global
- name
- owner_entity_id
- owner_department_id
- level_id
- direction_id
- cell_id
- created_at

space_resources
- space_id
- resource_id
- resource_type
- visibility
```

---

## 6. 资源对象

```sql
resources
- id
- type              -- file_system / database / knowledge_base / executor / agent_team / script_library
- name
- owning_space_id
- sensitivity_level
- created_at

file_objects
- resource_id
- storage_path
- version

database_objects
- resource_id
- database_uri
- schema_policy_id

knowledge_base_objects
- resource_id
- index_id
- embedding_policy_id

executor_objects
- resource_id
- executor_type
- sandbox_policy_id
```

---

## 7. 权限授权

```sql
permission_grants
- id
- issuer_entity_id
- subject_type       -- entity / department / role / agent_team
- subject_id
- resource_type
- resource_id
- action
- effect             -- allow / deny
- can_delegate
- expires_at
- created_at
```

---

## 8. 委托会话

```sql
delegated_permission_sessions
- id
- user_entity_id
- agent_team_id
- scope
- status
- created_at
- expires_at
```

---

## 9. 审计日志

```sql
audit_logs
- id
- actor_entity_id
- delegated_from_entity_id
- action
- resource_type
- resource_id
- decision
- reason
- risk_level
- created_at
```

---

## 10. 聊天、日程、任务

```sql
chat_threads
- id
- subject_entity_id
- target_object_type
- target_object_id
- thread_type
- status
- created_at

schedule_items
- id
- owner_entity_id
- title
- start_time
- end_time
- permission_session_id
- status

tasks
- id
- creator_entity_id
- executor_entity_id
- task_type
- permission_session_id
- status
- result_resource_id
- created_at
- updated_at
```

---

## 11. 数据模型原则

```text
1. 组织结构、空间结构、权限结构必须解耦。
2. 部门通过 department_cells 绑定多个格子。
3. 格子通过 org_cells 保证唯一占用。
4. 资源必须通过 space_resources 挂载到空间。
5. Agent Team 代用户执行必须产生 delegated_permission_sessions。
6. 所有执行行为必须进入 audit_logs。
```