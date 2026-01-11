heleny-gui\src\handle_resource.rs的ResourcePayload::Schedule { schedule } => {
                debug!("ResourcePayload::Schedule: {:?}", schedule);
            }部分会收到形如这样的日程，
{22737e10-0af2-4c79-8d37-a92be0755bfd: ScheduledTask { description: "去爬山", triggers: [Once { time: 2028-01-01T09:00:00+08:00 }], offset: 28800, next_trigger: Some(2028-01-01T09:00:00+08:00) }}

注意，你只负责slint的ui设计，rust的具体实现由我完成！
在heleny-gui\ui\schedule.slint里设计其ui，对应heleny-gui\ui\app.slint里面root.active-tab = 1; 的情况，要求：
1.足够圆角，id和description的展示布局类似task.slint，要展示下一次触发时间，位置就放id和description的下面，触发时间气泡的上面
2.然后列出各个触发时间，要包在胶囊里，时间要用方便的形式，比方Once { time: 2028-01-01T09:00:00+08:00 }就展示成“单次 2028-01-01 09:00:00”，其他几个展示形如“间隔 30 分钟，锚点 2026-01-01 09:00:00”，“每周一 11:00:00”，“每月 25 日 13:00:00”，“每日 08:00:00”。
3.每一条任务都要有1个按钮，用来取消日程，接受任务id为参数。
