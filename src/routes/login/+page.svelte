<script lang="ts">
  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';

  let email = '';
  let password = '';
  let error = '';
  let loading = false;

  async function handleLogin() {
    error = '';
    loading = true;

    try {
      await auth.login(email, password);
      goto('/editor');
    } catch (e) {
      error = 'Invalid credentials';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Login — Kashif</title>
</svelte:head>

<div class="login-page">
  <div class="login-container">
    <header class="login-header">
      <h1 class="login-title">Sign in</h1>
      <p class="login-desc">Access the resume editor</p>
    </header>

    <form on:submit|preventDefault={handleLogin} class="login-form">
      {#if error}
        <div class="error-msg">{error}</div>
      {/if}

      <div class="form-group">
        <label for="email" class="form-label">Email</label>
        <input
          type="email"
          id="email"
          bind:value={email}
          required
          class="form-input"
          placeholder="you@example.com"
          autocomplete="email"
        />
      </div>

      <div class="form-group">
        <label for="password" class="form-label">Password</label>
        <input
          type="password"
          id="password"
          bind:value={password}
          required
          class="form-input"
          placeholder="••••••••"
          autocomplete="current-password"
        />
      </div>

      <button type="submit" disabled={loading} class="submit-btn">
        {loading ? 'Signing in...' : 'Sign in'}
      </button>
    </form>

    <a href="/" class="back-link">← Back</a>
  </div>
</div>

<style>
  .login-page {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    background: var(--paper);
  }
  
  .login-container {
    width: 100%;
    max-width: 360px;
    animation: fade-up var(--dur-base) var(--ease-out-quart);
  }
  
  .login-header {
    text-align: center;
    margin-bottom: 2.5rem;
  }
  
  .login-title {
    font-size: 1.75rem;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 0.5rem;
  }
  
  .login-desc {
    font-size: 0.9375rem;
    color: var(--text-tertiary);
  }
  
  .login-form {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    padding: 2rem;
    background: var(--surface-raised);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
  }
  
  .error-msg {
    padding: 0.75rem 1rem;
    font-size: 0.875rem;
    color: var(--text-primary);
    background: var(--surface-sunken);
    border: 1px solid var(--text-faint);
    border-radius: var(--radius-sm);
    text-align: center;
  }
  
  .form-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .form-label {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-secondary);
  }
  
  .form-input {
    padding: 0.75rem 1rem;
    font-size: 0.9375rem;
    color: var(--text-primary);
    background: var(--paper);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    transition: border-color var(--dur-instant) var(--ease-out-quart);
  }
  
  .form-input::placeholder {
    color: var(--text-faint);
  }
  
  .form-input:focus {
    outline: none;
    border-color: var(--text-faint);
  }
  
  .submit-btn {
    padding: 0.875rem 1.5rem;
    font-size: 0.9375rem;
    font-weight: 500;
    color: var(--btn-primary-fg);
    background: var(--btn-primary-bg);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--dur-instant) var(--ease-out-quart);
    margin-top: 0.5rem;
  }
  
  .submit-btn:hover:not(:disabled) {
    background: var(--btn-primary-hover);
  }
  
  .submit-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .back-link {
    display: block;
    margin-top: 2rem;
    text-align: center;
    font-size: 0.875rem;
    color: var(--text-faint);
    transition: color var(--dur-instant) var(--ease-out-quart);
  }
  
  .back-link:hover {
    color: var(--text-primary);
  }
  
  @keyframes fade-up {
    from { opacity: 0; transform: translateY(20px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
