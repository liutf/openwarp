#[cfg(feature = "local_fs")]
use super::features::external_editor::ExternalEditorView;
use super::{
    settings_page::{
        build_sub_header, render_body_item, render_separator, Category, MatchData, PageType,
        SettingsPageMeta, SettingsPageViewHandle, SettingsWidget, HEADER_PADDING,
    },
    LocalOnlyIconState, SettingsAction, SettingsSection, ToggleState,
};
use crate::{
    appearance::Appearance, send_telemetry_from_ctx, settings::CodeSettings,
    terminal::general_settings::GeneralSettings, workspace::tab_settings::TabSettings,
    workspaces::update_manager::TeamUpdateManager, TelemetryEvent,
};
use ai::project_context::model::{ProjectContextModel, ProjectContextModelEvent};

use std::path::PathBuf;
use warp_core::{features::FeatureFlag, report_if_error, settings::ToggleableSetting as _};
use warpui::{
    elements::{ChildView, Container, Element, Empty, Flex, ParentElement},
    fonts::Weight,
    keymap::ContextPredicate,
    ui_components::{
        components::{UiComponent, UiComponentStyles},
        switch::SwitchStateHandle,
    },
    Action, AppContext, Entity, SingletonEntity, TypedActionView, View, ViewContext, ViewHandle,
};

/// Identifies which subpage of the Code settings the user is viewing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeSubpage {
    /// Codebase indexing and initialization settings.
    Indexing,
    /// External editor, code review panel, and project explorer settings.
    EditorAndCodeReview,
}

impl CodeSubpage {
    pub fn from_section(section: SettingsSection) -> Option<Self> {
        match section {
            SettingsSection::CodeIndexing => Some(Self::Indexing),
            SettingsSection::EditorAndCodeReview => Some(Self::EditorAndCodeReview),
            _ => None,
        }
    }

    pub fn title(&self) -> String {
        match self {
            Self::Indexing => crate::t!("settings-code-subpage-indexing-title"),
            Self::EditorAndCodeReview => crate::t!("settings-code-subpage-editor-review-title"),
        }
    }
}

pub struct CodeSettingsPageView {
    page: PageType<Self>,
    active_subpage: Option<CodeSubpage>,
    #[cfg(feature = "local_fs")]
    external_editor_view: Option<ViewHandle<ExternalEditorView>>,
}

impl CodeSettingsPageView {
    pub fn new(ctx: &mut ViewContext<CodeSettingsPageView>) -> Self {
        // 订阅 ProjectContextModel:project rules 变动时重渲染,
        // 让任何依赖 rule 集合的子页面/组件保持最新。
        ctx.subscribe_to_model(&ProjectContextModel::handle(ctx), |_me, _, event, ctx| {
            if matches!(event, ProjectContextModelEvent::KnownRulesChanged(_)) {
                ctx.notify();
            }
        });

        let code_page_widget = CodePageWidget;

        #[cfg(feature = "local_fs")]
        let external_editor_view;
        let page = if FeatureFlag::OpenWarpNewSettingsModes.is_enabled() {
            #[cfg(feature = "local_fs")]
            {
                external_editor_view = Some(ctx.add_typed_action_view(ExternalEditorView::new));
            }

            let codebase_indexing_widgets: Vec<Box<dyn SettingsWidget<View = Self>>> =
                vec![Box::new(CodebaseIndexingCategorizedWidget {
                    inner: code_page_widget,
                })];
            #[cfg(feature = "local_fs")]
            let mut code_editor_review_widgets: Vec<
                Box<dyn SettingsWidget<View = Self>>,
            > = vec![Box::new(ExternalEditorCodeWidget)];
            #[cfg(not(feature = "local_fs"))]
            let mut code_editor_review_widgets: Vec<
                Box<dyn SettingsWidget<View = Self>>,
            > = vec![];
            code_editor_review_widgets.extend([
                Box::new(AutoOpenCodeReviewPaneCodeWidget::default())
                    as Box<dyn SettingsWidget<View = Self>>,
                Box::new(CodeReviewPanelToggleWidget::default()),
                Box::new(CodeReviewDiffStatsToggleWidget::default()),
                Box::new(ProjectExplorerToggleWidget::default()),
                Box::new(GlobalSearchToggleWidget::default()),
            ]);
            let categories = vec![
                Category::new(
                    &*Box::leak(
                        crate::t!("settings-code-category-codebase-indexing").into_boxed_str(),
                    ),
                    codebase_indexing_widgets,
                ),
                Category::new(
                    &*Box::leak(crate::t!("settings-code-category-editor-review").into_boxed_str()),
                    code_editor_review_widgets,
                ),
            ];
            PageType::new_categorized(categories, None)
        } else {
            #[cfg(feature = "local_fs")]
            {
                external_editor_view = None;
            }
            let widgets: Vec<Box<dyn SettingsWidget<View = Self>>> =
                vec![Box::new(code_page_widget)];
            PageType::new_uncategorized(widgets, None)
        };

        Self {
            page,
            active_subpage: None,
            #[cfg(feature = "local_fs")]
            external_editor_view,
        }
    }

    /// 设置当前激活的子页面,并按需重建 page。
    pub fn set_active_subpage(
        &mut self,
        subpage: Option<CodeSubpage>,
        ctx: &mut ViewContext<Self>,
    ) {
        if self.active_subpage != subpage {
            self.active_subpage = subpage;
            // 按子页面的内容重建 widgets;subpage 为 None 时回到完整 categorized 页面。
            if let Some(subpage) = subpage {
                let mut widgets: Vec<Box<dyn SettingsWidget<View = Self>>> =
                    vec![Box::new(CodeSubpageHeaderWidget {
                        title: subpage.title(),
                    })];
                match subpage {
                    CodeSubpage::Indexing => {
                        widgets.push(Box::new(CodebaseIndexingCategorizedWidget {
                            inner: CodePageWidget,
                        }));
                    }
                    CodeSubpage::EditorAndCodeReview => {
                        #[cfg(feature = "local_fs")]
                        widgets.push(Box::new(ExternalEditorCodeWidget));
                        widgets.extend([
                            Box::new(AutoOpenCodeReviewPaneCodeWidget::default())
                                as Box<dyn SettingsWidget<View = Self>>,
                            Box::new(CodeReviewPanelToggleWidget::default()),
                            Box::new(CodeReviewDiffStatsToggleWidget::default()),
                            Box::new(ProjectExplorerToggleWidget::default()),
                            Box::new(GlobalSearchToggleWidget::default()),
                        ]);
                    }
                }
                // subpage widgets 自带 subheader 标题,所以这里不再传 page 级别的 title。
                self.page = PageType::new_uncategorized(widgets, None);
            } else {
                // None:重建完整的 categorized 页面(包含全部 widgets)。
                self.page = Self::build_full_page(ctx);
            }
            ctx.notify();
        }
    }

    /// 构建完整的 categorized 页面,用于默认/legacy 视图,以及搜索时回到全部 widgets 模式。
    fn build_full_page(_ctx: &mut ViewContext<Self>) -> PageType<Self> {
        if FeatureFlag::OpenWarpNewSettingsModes.is_enabled() {
            let code_page_widget = CodePageWidget;
            let codebase_indexing_widgets: Vec<Box<dyn SettingsWidget<View = Self>>> =
                vec![Box::new(CodebaseIndexingCategorizedWidget {
                    inner: code_page_widget,
                })];
            #[cfg(feature = "local_fs")]
            let mut code_editor_review_widgets: Vec<
                Box<dyn SettingsWidget<View = Self>>,
            > = vec![Box::new(ExternalEditorCodeWidget)];
            #[cfg(not(feature = "local_fs"))]
            let mut code_editor_review_widgets: Vec<
                Box<dyn SettingsWidget<View = Self>>,
            > = vec![];
            code_editor_review_widgets.extend([
                Box::new(AutoOpenCodeReviewPaneCodeWidget::default())
                    as Box<dyn SettingsWidget<View = Self>>,
                Box::new(CodeReviewPanelToggleWidget::default()),
                Box::new(CodeReviewDiffStatsToggleWidget::default()),
                Box::new(ProjectExplorerToggleWidget::default()),
                Box::new(GlobalSearchToggleWidget::default()),
            ]);
            let categories = vec![
                Category::new(
                    &*Box::leak(
                        crate::t!("settings-code-category-codebase-indexing").into_boxed_str(),
                    ),
                    codebase_indexing_widgets,
                ),
                Category::new(
                    &*Box::leak(crate::t!("settings-code-category-editor-review").into_boxed_str()),
                    code_editor_review_widgets,
                ),
            ];
            PageType::new_categorized(categories, None)
        } else {
            let widgets: Vec<Box<dyn SettingsWidget<View = Self>>> = vec![Box::new(CodePageWidget)];
            PageType::new_uncategorized(widgets, None)
        }
    }
}

impl Entity for CodeSettingsPageView {
    type Event = CodeSettingsPageEvent;
}

impl View for CodeSettingsPageView {
    fn ui_name() -> &'static str {
        "CodePage"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        self.page.render(self, app)
    }
}

#[derive(Debug, Clone)]
pub enum CodeSettingsPageEvent {
    SignupAnonymousUser,
    OpenProjectRules { rule_paths: Vec<PathBuf> },
}

// Code 页面的 actions 定义。
#[derive(Debug, Clone)]
pub enum CodeSettingsPageAction {
    SignupAnonymousUser,
    OpenProjectRules { rule_paths: Vec<PathBuf> },
    ToggleCodeReviewPanel,
    ToggleShowCodeReviewDiffStats,
    ToggleAutoOpenCodeReviewPane,
    ToggleProjectExplorer,
    ToggleGlobalSearch,
}

impl TypedActionView for CodeSettingsPageView {
    type Action = CodeSettingsPageAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            CodeSettingsPageAction::SignupAnonymousUser => {
                ctx.emit(CodeSettingsPageEvent::SignupAnonymousUser);
            }
            CodeSettingsPageAction::OpenProjectRules { rule_paths } => {
                ctx.emit(CodeSettingsPageEvent::OpenProjectRules {
                    rule_paths: rule_paths.clone(),
                });
            }
            CodeSettingsPageAction::ToggleCodeReviewPanel => {
                TabSettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(settings.show_code_review_button.toggle_and_save_value(ctx));
                });
                ctx.notify();
            }
            CodeSettingsPageAction::ToggleShowCodeReviewDiffStats => {
                TabSettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(settings
                        .show_code_review_diff_stats
                        .toggle_and_save_value(ctx));
                });
                ctx.notify();
            }
            CodeSettingsPageAction::ToggleProjectExplorer => {
                CodeSettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(settings.show_project_explorer.toggle_and_save_value(ctx));
                });
                ctx.notify();
            }
            CodeSettingsPageAction::ToggleGlobalSearch => {
                CodeSettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(settings.show_global_search.toggle_and_save_value(ctx));
                });
                ctx.notify();
            }
            CodeSettingsPageAction::ToggleAutoOpenCodeReviewPane => {
                GeneralSettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(settings
                        .auto_open_code_review_pane_on_first_agent_change
                        .toggle_and_save_value(ctx));
                });
                send_telemetry_from_ctx!(
                    TelemetryEvent::FeaturesPageAction {
                        action: "ToggleAutoOpenCodeReviewPane".to_string(),
                        value: format!(
                            "{}",
                            *GeneralSettings::as_ref(ctx)
                                .auto_open_code_review_pane_on_first_agent_change
                        )
                    },
                    ctx
                );
                ctx.notify();
            }
        }
    }
}

pub fn init_actions_from_parent_view<T: Action + Clone>(
    _app: &mut AppContext,
    _context: &ContextPredicate,
    _builder: fn(SettingsAction) -> T,
) {
}

struct CodePageWidget;

impl SettingsWidget for CodePageWidget {
    type View = CodeSettingsPageView;

    fn search_terms(&self) -> &str {
        "code coding project rules"
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        _app: &AppContext,
    ) -> Box<dyn Element> {
        let mut content = Flex::column();

        content.add_child(self.render_code_header(appearance));
        content.add_child(render_separator(appearance));

        Container::new(content.finish())
            .with_uniform_padding(24.0)
            .finish()
    }
}

impl CodePageWidget {
    /// 渲染主标题 "Code"。
    fn render_code_header(&self, appearance: &Appearance) -> Box<dyn Element> {
        let ui_builder = appearance.ui_builder();
        let theme = appearance.theme();

        Container::new(
            ui_builder
                .span(crate::t!("settings-code-feature-name"))
                .with_style(UiComponentStyles {
                    font_size: Some(24.0),
                    font_weight: Some(Weight::Bold),
                    font_color: Some(theme.active_ui_text_color().into()),
                    ..Default::default()
                })
                .build()
                .finish(),
        )
        .with_padding_bottom(15.)
        .finish()
    }
}

/// 简单的子页面标题 widget,只渲染 subheader 文本。
struct CodeSubpageHeaderWidget {
    title: String,
}

impl SettingsWidget for CodeSubpageHeaderWidget {
    type View = CodeSettingsPageView;

    fn search_terms(&self) -> &str {
        &self.title
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        _app: &AppContext,
    ) -> Box<dyn Element> {
        build_sub_header(appearance, self.title.clone(), None)
            .with_padding_bottom(HEADER_PADDING)
            .finish()
    }
}

/// LSP 子系统已下线,该 widget 目前没有可渲染的 codebase indexing 内容;
/// 保留它是为了让 page/category 结构以及 subpage 路由继续可用,
/// 等后续有新的"项目规则 / 仓库索引"内容时再填回来。
struct CodebaseIndexingCategorizedWidget {
    inner: CodePageWidget,
}

impl SettingsWidget for CodebaseIndexingCategorizedWidget {
    type View = CodeSettingsPageView;

    fn search_terms(&self) -> &str {
        "repository code path project rules"
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        // 仍把渲染委托给 inner 的 header 部分,避免 inner 字段成为死码,
        // 也让用户在该子页面下能看到一致的 "Code" 标题与分隔线。
        self.inner.render(view, appearance, app)
    }
}

#[cfg(feature = "local_fs")]
struct ExternalEditorCodeWidget;

#[cfg(feature = "local_fs")]
impl SettingsWidget for ExternalEditorCodeWidget {
    type View = CodeSettingsPageView;

    fn search_terms(&self) -> &str {
        "code editor open files markdown AI conversations layout pane tab"
    }

    fn render(
        &self,
        view: &Self::View,
        _appearance: &Appearance,
        _app: &AppContext,
    ) -> Box<dyn Element> {
        if let Some(editor_view) = &view.external_editor_view {
            ChildView::new(editor_view).finish()
        } else {
            Empty::new().finish()
        }
    }
}

#[derive(Default)]
struct AutoOpenCodeReviewPaneCodeWidget {
    switch_state: SwitchStateHandle,
}

impl SettingsWidget for AutoOpenCodeReviewPaneCodeWidget {
    type View = CodeSettingsPageView;

    fn search_terms(&self) -> &str {
        "oz auto open code review pane panel agent mode change first time accepted diff view conversation"
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let general_settings = GeneralSettings::as_ref(app);
        render_body_item::<CodeSettingsPageAction>(
            crate::t!("settings-code-auto-open-review-panel").into(),
            None,
            LocalOnlyIconState::Hidden,
            ToggleState::Enabled,
            appearance,
            appearance
                .ui_builder()
                .switch(self.switch_state.clone())
                .check(*general_settings.auto_open_code_review_pane_on_first_agent_change)
                .build()
                .on_click(move |ctx, _, _| {
                    ctx.dispatch_typed_action(CodeSettingsPageAction::ToggleAutoOpenCodeReviewPane);
                })
                .finish(),
            Some(crate::t!("settings-code-auto-open-review-panel-desc").into()),
        )
    }
}

impl SettingsPageMeta for CodeSettingsPageView {
    fn section() -> SettingsSection {
        SettingsSection::Code
    }

    fn update_filter(&mut self, query: &str, ctx: &mut ViewContext<Self>) -> MatchData {
        self.page.update_filter(query, ctx)
    }

    fn should_render(&self, _ctx: &AppContext) -> bool {
        FeatureFlag::OpenWarpNewSettingsModes.is_enabled()
    }

    fn on_page_selected(&mut self, _: bool, ctx: &mut ViewContext<Self>) {
        // 立即拉一次 workspace metadata,而不是等下一次轮询,
        // 让用户能更快看到自己是否处于一个 workspace 中。
        std::mem::drop(
            TeamUpdateManager::handle(ctx)
                .update(ctx, |manager, ctx| manager.refresh_workspace_metadata(ctx)),
        );
    }

    fn scroll_to_widget(&mut self, widget_id: &'static str) {
        self.page.scroll_to_widget(widget_id)
    }

    fn clear_highlighted_widget(&mut self) {
        self.page.clear_highlighted_widget();
    }
}

impl From<ViewHandle<CodeSettingsPageView>> for SettingsPageViewHandle {
    fn from(view_handle: ViewHandle<CodeSettingsPageView>) -> Self {
        SettingsPageViewHandle::Code(view_handle)
    }
}

#[derive(Default)]
struct CodeReviewPanelToggleWidget {
    switch_state: SwitchStateHandle,
}

impl SettingsWidget for CodeReviewPanelToggleWidget {
    type View = CodeSettingsPageView;

    fn search_terms(&self) -> &str {
        "code review panel right side diff git"
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let tab_settings = TabSettings::as_ref(app);

        render_body_item::<CodeSettingsPageAction>(
            crate::t!("settings-code-show-code-review-button").into(),
            None,
            LocalOnlyIconState::Hidden,
            ToggleState::Enabled,
            appearance,
            appearance
                .ui_builder()
                .switch(self.switch_state.clone())
                .check(*tab_settings.show_code_review_button)
                .build()
                .on_click(move |ctx, _, _| {
                    ctx.dispatch_typed_action(CodeSettingsPageAction::ToggleCodeReviewPanel);
                })
                .finish(),
            Some(crate::t!("settings-code-show-code-review-button-desc").into()),
        )
    }
}

#[derive(Default)]
struct CodeReviewDiffStatsToggleWidget {
    switch_state: SwitchStateHandle,
}

impl SettingsWidget for CodeReviewDiffStatsToggleWidget {
    type View = CodeSettingsPageView;

    fn search_terms(&self) -> &str {
        "code review diff stats lines added removed counts"
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let tab_settings = TabSettings::as_ref(app);

        render_body_item::<CodeSettingsPageAction>(
            crate::t!("settings-code-show-diff-stats").into(),
            None,
            LocalOnlyIconState::Hidden,
            ToggleState::Enabled,
            appearance,
            appearance
                .ui_builder()
                .switch(self.switch_state.clone())
                .check(*tab_settings.show_code_review_diff_stats)
                .build()
                .on_click(move |ctx, _, _| {
                    ctx.dispatch_typed_action(
                        CodeSettingsPageAction::ToggleShowCodeReviewDiffStats,
                    );
                })
                .finish(),
            Some(crate::t!("settings-code-show-diff-stats-desc").into()),
        )
    }
}

#[derive(Default)]
struct ProjectExplorerToggleWidget {
    switch_state: SwitchStateHandle,
}

impl SettingsWidget for ProjectExplorerToggleWidget {
    type View = CodeSettingsPageView;

    fn search_terms(&self) -> &str {
        "project explorer file tree left panel tools"
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let code_settings = CodeSettings::as_ref(app);

        render_body_item::<CodeSettingsPageAction>(
            crate::t!("settings-code-project-explorer").into(),
            None,
            LocalOnlyIconState::Hidden,
            ToggleState::Enabled,
            appearance,
            appearance
                .ui_builder()
                .switch(self.switch_state.clone())
                .check(*code_settings.show_project_explorer)
                .build()
                .on_click(move |ctx, _, _| {
                    ctx.dispatch_typed_action(CodeSettingsPageAction::ToggleProjectExplorer);
                })
                .finish(),
            Some(crate::t!("settings-code-project-explorer-desc").into()),
        )
    }
}

#[derive(Default)]
struct GlobalSearchToggleWidget {
    switch_state: SwitchStateHandle,
}

impl SettingsWidget for GlobalSearchToggleWidget {
    type View = CodeSettingsPageView;

    fn search_terms(&self) -> &str {
        "global search file search left panel tools"
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let code_settings = CodeSettings::as_ref(app);

        render_body_item::<CodeSettingsPageAction>(
            crate::t!("settings-code-global-search").into(),
            None,
            LocalOnlyIconState::Hidden,
            ToggleState::Enabled,
            appearance,
            appearance
                .ui_builder()
                .switch(self.switch_state.clone())
                .check(*code_settings.show_global_search)
                .build()
                .on_click(move |ctx, _, _| {
                    ctx.dispatch_typed_action(CodeSettingsPageAction::ToggleGlobalSearch);
                })
                .finish(),
            Some(crate::t!("settings-code-global-search-desc").into()),
        )
    }
}
