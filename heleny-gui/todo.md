heleny-gui\src\handle_resource.rs的ResourcePayload::ToolAbstracts { abstracts }=>{
                debug!("ToolAbstracts: {:?}",abstracts);
            }部分会收到形如这样的工具摘要，
Vec<ToolAbstract>
pub struct ToolAbstract {
    pub name: String,
    pub description: String,
    pub commands: HashMap<String,String>,
    pub available: bool,
}
commands的key是名字，value是描述

注意，你只负责slint的ui设计，rust的具体实现由我完成！
在heleny-gui\ui\tools.slint里设计其ui，对应heleny-gui\ui\app.slint里面root.active-tab = 5; 的情况，要求：
1.足够圆角，布局类似task.slint，默认展示name和description，而要点击这个条目后触发下拉才展示commands信息。展示名字（name，commands的key）的时候要放在胶囊里。
2.条目右上角有个实心小圆，如果available=true则亮绿色，否则亮红色
