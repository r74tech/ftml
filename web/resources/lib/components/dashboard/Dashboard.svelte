<script lang="ts" context="module">
  import { route } from "@wikijump/api"

  const DASHBOARD_ROUTE = route("dashboard")

  /**
   * Returns a dashboard route.
   *
   * @param path - The subpath.
   */
  export function dashboardRoute(path = "") {
    if (path.startsWith("/")) path = path.substring(1)
    return `${DASHBOARD_ROUTE}/${path}`
  }
</script>

<script lang="ts">
  import Locale from "@wikijump/fluent"
  import { Route } from "tinro"
  import RouteAnnouncer from "../RouteAnnouncer.svelte"
  import UserProfile from "../UserProfile.svelte"
  import DashboardLink from "./DashboardLink.svelte"
  import DashboardPanel from "./DashboardPanel.svelte"
  import SettingsPanel from "./panels/SettingsPanel.svelte"

  const t = Locale.makeComponentFormatter("dashboard")

  // bit of a hack, unfortunately, but this is to prevent Tinro
  // from hijacking non-dashboard routes

  const selector = `a:not([data-tinro-ignore]):not([href^="${dashboardRoute()}"])`

  const observer = new MutationObserver(() => {
    const links = Array.from(document.body.querySelectorAll<HTMLAnchorElement>(selector))
    for (const link of links) {
      link.setAttribute("data-tinro-ignore", "")
    }
  })

  observer.observe(document.body, { childList: true, subtree: true })
</script>

<div class="dashboard">
  <RouteAnnouncer />

  <ul class="dashboard-links">
    <DashboardLink path="profile">{$t("profile")}</DashboardLink>

    <DashboardLink path="notifications">{$t("notifications")}</DashboardLink>

    <DashboardLink
      path="messages"
      subpaths={[
        { path: "inbox", title: $t("inbox") },
        { path: "sent", title: $t("sent") },
        { path: "invitations", title: $t("invitations") },
        { path: "applications", title: $t("applications") }
      ]}
    >
      {$t("messages")}
    </DashboardLink>

    <DashboardLink
      path="settings"
      subpaths={[
        { path: "profile", title: $t("profile") },
        { path: "account", title: $t("account") }
      ]}
    >
      {$t("settings")}
    </DashboardLink>
  </ul>

  <div class="dashboard-panels">
    <Route path={dashboardRoute("*")} firstmatch>
      <Route fallback redirect={dashboardRoute("profile")} />

      <DashboardPanel path="/profile" title={$t("profile")}>
        <UserProfile />
      </DashboardPanel>

      <!-- TODO: notifications -->
      <DashboardPanel path="/notifications/*" title={$t("notifications")} />

      <!-- TODO: messages/inbox -->
      <DashboardPanel path="/messages/*" title={$t("messages")} />

      <DashboardPanel path="/settings/*" title={$t("settings")}>
        <SettingsPanel />
      </DashboardPanel>
    </Route>
  </div>
</div>

<style global lang="scss">
  #main > wj-component-loader[ld-load="Dashboard"] {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .dashboard {
    display: flex;
    min-height: 100%;
    contain: layout;

    @include media("<=small") {
      flex-direction: column;
    }
  }

  .dashboard-links {
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    row-gap: 0.5rem;
    min-width: 16rem;
    padding-right: 1rem;
    margin-right: 2rem;
    list-style: none;
    border-right: solid 0.125rem var(--col-border);

    @include media("<=small") {
      padding-right: 0;
      padding-bottom: 1rem;
      margin-right: 0;
      margin-bottom: 2rem;
      border-right: none;
      border-bottom: solid 0.125rem var(--col-border);
    }
  }

  .dashboard-panels {
    flex-grow: 1;

    @include media("<=small") {
      padding: 0 1rem;
    }
  }
</style>
