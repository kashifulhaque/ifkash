# [**Kashiful Haque**](https://ifkash.dev)

### **Tech stack** 📚
- [Cloudflare Workers](https://workers.cloudflare.com)
- [Cloudflare Pages](https://pages.cloudflare.com)
- [Mise](https://mise.jdx.dev)
  - [Rust](https://rust-lang.org)
  - [Bun](https://bun.sh)
    - [SvelteKit](https://kit.svelte.dev)

### **To run locally**

```sh
# Install mise (if not already installed)
curl https://mise.run | sh

# Activate tools and prep once
mise run api:prep

# Start the worker
mise run api:dev

# To run SvelteKit app
mise run web:dev
```
