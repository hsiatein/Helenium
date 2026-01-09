heleny-gui\src\lib.rs的ResourcePayload::TaskAbstract { task_abstracts }=>{
                    debug!("任务摘要: {:?}",task_abstracts)
                }部分会收到形如这样的任务摘要，
[TaskAbstract { id: 7135f2ff-0571-45c8-a7ba-03748c3238a5, task_description: "从交换目录中获取图片文件并发送给用户HT", status: Success }]

注意，你只负责slint的ui设计，rust的具体实现由我完成！
在heleny-gui/ui/tasks.slint里设计其ui，对应root.active-tab = 3; 的情况，要求：
1.足够圆角，id展示要包在圆圈里, status展示要包在颜色胶囊里，Success绿色，Fail红色，Running蓝色，Pending灰色，Canceled黄色。
2.每一条任务都要有1个按钮，用来取消任务，接受任务id为参数。
3.然后每个任务都要预留Vec<String>用来装任务日志，默认不显示日志只显示task_description，但是任务词条本身要可以点击，接受任务id为参数，用来切换“下拉显示日志/收回不显示日志”。搞完生成一些slint里面的例子给我用来测试ui布局。