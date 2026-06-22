# 03. 组织权限图

> Version: 0.1  
> Parent: [GESA 企业空间系统](./README.md)

---

## 1. 定义

组织权限图用于可视化管理企业权限系统。

基础维度：

```text
Level      层级
Direction  方向
Cell       格子 = Level × Direction
```

部门由多个格子的集合定义：

```text
Department = Set<Cell>
```

---

## 2. 最高权限规则

```text
1. 最高权限管理器默认归属 CEO。
2. CEO 可以把最高权限管理器授权给其他对象。
3. 只有 CEO 拥有 CEO 任命权。
4. CEO 必须是 HumanEntity。
5. 最高权限管理器负责管理部门系统、层级系统和方向系统。
```

---

## 3. 层级 Level

层级表示企业中的管理高度。

示例：

```text
7. CEO / 总管理层
6. 总办公室 / 总监层
5. 中台管理层
4. 部门管理层
3. 小组管理层
2. 高级执行层
1. 基层执行层
```

---

## 4. 方向 Direction

方向表示业务纵条。

示例：

```text
1. 财务方向
2. 税务方向
3. 法律方向
4. 采购方向
5. 仓库方向
6. 客户管理方向
7. 市场分析方向
8. 研发方向
9. 运营方向
10. 人力资源方向
11. Agent 资源方向
```

---

## 5. 格子 Cell

格子由层级和方向构成：

```text
Cell(level, direction)
```

可以写成：

```text
7·1
5·6
3·2
```

硬规则：

```text
同一个 Cell(level, direction) 只能被一个部门占用。
```

---

## 6. 部门 Department

部门是格子的集合。

示例：

```text
管理办公室 = {7·1, 7·2, 7·3, 7·4, 7·5, 7·6, 7·7, 7·8, 7·9, 7·10}
财务总办公室 = {6·1, 6·2, 6·3}
采购总办公室 = {6·4, 6·5, 6·1}
```

这允许一个部门跨多个方向。

---

## 7. 部门向下投影

部门可以管理其占据格子向下投影覆盖的格子空间。

```text
Coverage(Department A) =
for each Cell(level = L, direction = D) in A.cells:
    include all Cell(level <= L, direction = D)
```

示例：

```text
A部门 = {5·1, 5·2, 5·4, 5·6}
```

则 A 部门覆盖：

```text
1-5 层级的 1、2、4、6 四个方向
```

共 20 个格子空间。

---

## 8. 管理下级部门空间

部门 A 可以管理部门 B 的部门空间，当且仅当 B 被 A 完全笼罩。

```text
CanManageDepartmentSpace(A, B) = B.cells ⊆ Coverage(A)
```

示例：

```text
A部门 = {5·1, 5·2, 5·4, 5·6}
B部门 = {4·1, 4·6}
C部门 = {3·2, 3·3}
```

判断：

```text
B.cells 完全包含在 Coverage(A) 中，所以 A 可以管理 B 的部门空间。
C 中的 3·3 不在 Coverage(A) 中，所以 A 不能完整管理 C 的部门空间。
```

---

## 9. 部门首脑

```text
1. 每个部门必须有一个明确的首脑对象。
2. 部门首脑对本部门拥有全部管理权限。
3. 部门首脑可以是 HumanEntity，也可以是 AgentEntity。
4. AgentEntity 作为部门首脑时，管理行为必须进入审计日志。
```