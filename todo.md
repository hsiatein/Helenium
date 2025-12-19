# heleny-kernel
service trait包含：
依赖，启动（返回句柄），句柄相关操作（如关闭，发送消息）

services:
- file_service
- log_service
- kernel_service
- scheduler_service
- heleny_service
- task_service
- frontend_service(签名验证)
- network_service
- http_service
- shell_service
- memory_service
- db_service