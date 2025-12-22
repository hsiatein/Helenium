# heleny-kernel
service trait包含：
依赖，启动（返回句柄），句柄相关操作（如关闭，发送消息）

services:
- [ ] file_service
- [ ] log_service
- [ ] kernel_service
- [ ] scheduler_service
- [ ] chat_service
- [ ] task_service
- [ ] frontend_service(签名验证)
- [ ] network_service
- [ ] http_service
- [ ] shell_service
- [ ] memory_service
- [ ] db_service
- [ ] config_service
- [ ] auth_service

todo:
- [ ] 1.token换成每人都有，权限不同
- [ ] 2.中转时token换name
- [ ] 3.汇报是否存活
- [x] 4.内核和内核代行者共享字段
- [x] 5.tracing日志