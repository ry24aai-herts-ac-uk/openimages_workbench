<script lang="ts">
  import { page } from '$app/stores';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();

  const navItems = [
    { href: '/', label: 'Home', icon: '🏠' },
    { href: '/download', label: 'Download', icon: '⬇️' },
    { href: '/jobs', label: 'Jobs', icon: '📋' },
    { href: '/viewer', label: 'Viewer', icon: '🖼️' },
  ];
</script>

<div class="app">
  <nav class="sidebar">
    <div class="sidebar-header">
      <span class="logo-icon">🔬</span>
      <span class="logo-text">Open Images<br />Workbench</span>
    </div>
    <ul class="nav-list">
      {#each navItems as item}
        <li>
          <a
            href={item.href}
            class:active={$page.url.pathname === item.href}
          >
            <span class="nav-icon">{item.icon}</span>
            <span class="nav-label">{item.label}</span>
          </a>
        </li>
      {/each}
    </ul>
    <div class="sidebar-footer">
      <span class="version">v0.1.0 – Phase 1</span>
    </div>
  </nav>

  <main class="content">
    {@render children()}
  </main>
</div>

<style>
  :global(*) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
    background: #0f1117;
    color: #e2e8f0;
  }

  .app {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  /* ---- Sidebar ---- */
  .sidebar {
    width: 220px;
    min-width: 220px;
    background: #161b27;
    border-right: 1px solid #2d3748;
    display: flex;
    flex-direction: column;
    padding: 0;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 20px 16px 16px;
    border-bottom: 1px solid #2d3748;
  }

  .logo-icon {
    font-size: 1.8rem;
    line-height: 1;
  }

  .logo-text {
    font-size: 0.85rem;
    font-weight: 700;
    line-height: 1.3;
    color: #90cdf4;
    letter-spacing: 0.01em;
  }

  .nav-list {
    list-style: none;
    padding: 12px 8px;
    flex: 1;
  }

  .nav-list li {
    margin-bottom: 2px;
  }

  .nav-list a {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 12px;
    border-radius: 8px;
    text-decoration: none;
    color: #a0aec0;
    font-size: 0.9rem;
    font-weight: 500;
    transition: background 0.15s, color 0.15s;
  }

  .nav-list a:hover {
    background: #2d3748;
    color: #e2e8f0;
  }

  .nav-list a.active {
    background: #2b4b7e;
    color: #90cdf4;
  }

  .nav-icon {
    font-size: 1.1rem;
    width: 22px;
    text-align: center;
  }

  .sidebar-footer {
    padding: 12px 16px;
    border-top: 1px solid #2d3748;
  }

  .version {
    font-size: 0.7rem;
    color: #4a5568;
  }

  /* ---- Content area ---- */
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 32px;
    background: #0f1117;
  }
</style>
