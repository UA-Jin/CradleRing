# 工作流指南

CradleRing 工作流引擎支持 9 种节点类型：

1. **LLM** - 调用大语言模型
2. **Tool** - 执行单个工具
3. **Agent** - 运行角色化 Agent
4. **Condition** - 条件分支
5. **Parallel** - 并行扇出
6. **Interrupt** - 暂停等待人工输入
7. **HumanReview** - 暂停等待人工审核
8. **End** - 终止

支持检查点、回滚、interrupt 断点。
