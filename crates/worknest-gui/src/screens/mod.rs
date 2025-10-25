//! Screen definitions

use worknest_core::models::{ProjectId, TicketId};

pub mod dashboard;
pub mod login;
pub mod project_detail;
pub mod project_list;
pub mod register;
pub mod ticket_board;
pub mod ticket_detail;
pub mod ticket_list;

pub use dashboard::DashboardScreen;
pub use login::LoginScreen;
pub use project_detail::ProjectDetailScreen;
pub use project_list::ProjectListScreen;
pub use register::RegisterScreen;
pub use ticket_board::TicketBoardScreen;
pub use ticket_detail::TicketDetailScreen;
pub use ticket_list::TicketListScreen;

/// Application screens
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    /// Login screen
    Login,
    /// Registration screen
    Register,
    /// Dashboard (home)
    Dashboard,
    /// Project list
    ProjectList,
    /// Project detail view
    ProjectDetail(ProjectId),
    /// Ticket list view
    TicketList { project_id: Option<ProjectId> },
    /// Ticket board (Kanban) view
    TicketBoard { project_id: ProjectId },
    /// Ticket detail view
    TicketDetail(TicketId),
    /// Settings
    Settings,
}
