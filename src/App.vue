<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  getDesktopInfo,
  greetCreator,
  type DesktopInfo,
} from "./lib/desktop";

type View = "dashboard" | "templates" | "history" | "settings";

interface NavItem {
  id: View;
  label: string;
  icon: string;
}

const navItems: NavItem[] = [
  { id: "dashboard", label: "Overview", icon: "✦" },
  { id: "templates", label: "Templates", icon: "▦" },
  { id: "history", label: "History", icon: "◷" },
  { id: "settings", label: "Settings", icon: "⚙" },
];

const activeView = ref<View>("dashboard");
const desktopInfo = ref<DesktopInfo | null>(null);
const welcomeMessage = ref("Your creative workspace is ready.");

const currentView = computed(
  () => navItems.find((item) => item.id === activeView.value) ?? navItems[0],
);

const runtimeLabel = computed(() =>
  desktopInfo.value ? "Tauri desktop" : "Web preview",
);

const runtimeDetail = computed(() =>
  desktopInfo.value
    ? desktopInfo.value.platform + " · v" + desktopInfo.value.version
    : "Run pnpm tauri:dev for native shell",
);

const pageMeta = computed(() => {
  const metadata: Record<
    Exclude<View, "dashboard">,
    { title: string; description: string }
  > = {
    templates: {
      title: "Template library",
      description:
        "A focused home for reusable formats, brand presets, and quick-start ideas.",
    },
    history: {
      title: "Your history",
      description:
        "Recent exports and drafts will live here as the editor takes shape.",
    },
    settings: {
      title: "Workspace settings",
      description:
        "Configure storage, appearance, shortcuts, and future AI providers in one place.",
    },
  };

  return activeView.value === "dashboard"
    ? { title: "Overview", description: "" }
    : metadata[activeView.value];
});

function selectView(view: View) {
  activeView.value = view;
}

async function loadDesktopStatus() {
  try {
    desktopInfo.value = await getDesktopInfo();
    welcomeMessage.value = await greetCreator("Creator");
  } catch {
    desktopInfo.value = null;
    welcomeMessage.value =
      "Preview mode is ready. Connect the desktop shell when you are ready.";
  }
}

onMounted(loadDesktopStatus);
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand">
        <div class="brand-mark">M</div>
        <div>
          <strong>Meme Studio</strong>
          <span>Creator workspace</span>
        </div>
      </div>

      <button
        class="workspace-switcher"
        type="button"
        aria-label="Switch workspace"
      >
        <span class="workspace-avatar">MS</span>
        <span class="workspace-copy">
          <strong>Personal workspace</strong>
          <small>Local project</small>
        </span>
        <span class="workspace-chevron">⌄</span>
      </button>

      <nav class="primary-nav" aria-label="Primary navigation">
        <p class="nav-label">Workspace</p>
        <button
          v-for="item in navItems"
          :key="item.id"
          class="nav-item"
          :class="{ active: activeView === item.id }"
          type="button"
          @click="selectView(item.id)"
        >
          <span class="nav-glyph" aria-hidden="true">{{ item.icon }}</span>
          <span>{{ item.label }}</span>
          <span v-if="item.id === 'history'" class="nav-badge">0</span>
        </button>
      </nav>

      <div class="sidebar-footer">
        <div class="health-card">
          <span class="status-dot" aria-hidden="true"></span>
          <div>
            <strong>{{ runtimeLabel }}</strong>
            <small>{{ runtimeDetail }}</small>
          </div>
        </div>

        <div class="user-card">
          <div class="user-avatar">YC</div>
          <div>
            <strong>Your account</strong>
            <small>Free workspace</small>
          </div>
          <span class="user-more">•••</span>
        </div>
      </div>
    </aside>

    <section class="main-column">
      <header class="topbar">
        <div class="breadcrumb">
          <span>Workspace</span>
          <span class="breadcrumb-separator">/</span>
          <strong>{{ currentView.label }}</strong>
        </div>
        <div class="topbar-actions">
          <button
            class="shortcut-button"
            type="button"
            title="Keyboard shortcuts"
          >
            ⌘ K
          </button>
          <button class="upgrade-button" type="button">
            Upgrade <span aria-hidden="true">↗</span>
          </button>
        </div>
      </header>

      <main class="content">
        <section v-if="activeView === 'dashboard'" class="dashboard-view">
          <div class="hero-card">
            <div class="hero-copy">
              <p class="eyebrow">Meme Studio · v0.1</p>
              <h1>Make the idea<br /><em>unmissable.</em></h1>
              <p class="hero-description">
                A calm, local-first workspace for turning a spark into
                something people want to share.
              </p>
              <div class="hero-actions">
                <button
                  class="primary-button"
                  type="button"
                  @click="selectView('templates')"
                >
                  Create a meme <span aria-hidden="true">→</span>
                </button>
                <button
                  class="secondary-button"
                  type="button"
                  @click="selectView('templates')"
                >
                  Browse templates
                </button>
              </div>
              <p class="system-note">
                <span class="sparkle" aria-hidden="true">✦</span
                >{{ welcomeMessage }}
              </p>
            </div>
            <div class="hero-art" aria-hidden="true">
              <div class="art-orbit orbit-one"></div>
              <div class="art-orbit orbit-two"></div>
              <div class="art-card">
                <span class="art-card-label">IDEA / 001</span>
                <span class="art-card-title">Make it<br /><strong>memorable.</strong></span>
                <span class="art-card-footer">local draft <b>↗</b></span>
              </div>
              <span class="art-star star-one">✦</span>
              <span class="art-star star-two">✦</span>
              <span class="art-dot dot-one"></span>
              <span class="art-dot dot-two"></span>
            </div>
          </div>

          <div class="section-heading">
            <div>
              <p class="eyebrow">At a glance</p>
              <h2>Your creative pulse</h2>
            </div>
            <span class="section-caption">This workspace · All time</span>
          </div>

          <div class="stats-grid">
            <article class="stat-card">
              <div class="stat-icon stat-icon-violet">✦</div>
              <div>
                <span class="stat-label">Created</span>
                <strong>0</strong>
              </div>
              <span class="stat-trend">Start here</span>
            </article>
            <article class="stat-card">
              <div class="stat-icon stat-icon-orange">↗</div>
              <div>
                <span class="stat-label">Shared</span>
                <strong>0</strong>
              </div>
              <span class="stat-trend muted">Soon</span>
            </article>
            <article class="stat-card">
              <div class="stat-icon stat-icon-blue">◷</div>
              <div>
                <span class="stat-label">Drafts</span>
                <strong>0</strong>
              </div>
              <span class="stat-trend muted">Empty</span>
            </article>
          </div>

          <div class="section-heading recent-heading">
            <div>
              <p class="eyebrow">Keep moving</p>
              <h2>Recent projects</h2>
            </div>
            <button
              class="text-button"
              type="button"
              @click="selectView('history')"
            >
              View history <span>→</span>
            </button>
          </div>

          <div class="recent-card">
            <div class="empty-icon">✧</div>
            <h3>Your next favorite meme starts here.</h3>
            <p>
              Pick a template or start with a blank canvas. Your drafts stay
              local while the idea gets good.
            </p>
            <button
              class="secondary-button"
              type="button"
              @click="selectView('templates')"
            >
              Explore templates
            </button>
          </div>
        </section>

        <section v-else class="placeholder-view">
          <div class="placeholder-icon" aria-hidden="true">
            {{ currentView.icon }}
          </div>
          <p class="eyebrow">Meme Studio · Foundation</p>
          <h1>{{ pageMeta.title }}</h1>
          <p>{{ pageMeta.description }}</p>
          <button
            class="primary-button"
            type="button"
            @click="selectView('dashboard')"
          >
            Back to overview <span aria-hidden="true">→</span>
          </button>
        </section>

        <footer class="content-footer">
          <span>Meme Studio</span>
          <span>Built for local-first creativity</span>
          <span class="footer-status"><i></i> Foundation ready</span>
        </footer>
      </main>
    </section>
  </div>
</template>
