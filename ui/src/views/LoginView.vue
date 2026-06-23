<script setup lang="ts">
import {onMounted, ref} from "vue";
import {useRoute, useRouter} from "vue-router";
import {getSession, login} from "../network/api";

const route = useRoute();
const router = useRouter();
const password = ref("");
const loading = ref(false);
const errorMessage = ref("");
const authConfigured = ref(true);

const redirectPath = () => {
  const redirect = route.query.redirect;
  return typeof redirect === "string" && redirect.startsWith("/") ? redirect : "/";
}

onMounted(async () => {
  try {
    const session = await getSession();
    authConfigured.value = session.authConfigured;
    if (session.authenticated) {
      await router.replace(redirectPath());
    }
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : "读取会话失败";
  }
})

const submit = async () => {
  if (!password.value || loading.value) return;
  loading.value = true;
  errorMessage.value = "";
  try {
    await login(password.value);
    await router.replace(redirectPath());
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : "登录失败";
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="login-page">
    <form class="login-panel" @submit.prevent="submit">
      <div>
        <h1>Web File Browser</h1>
        <p>管理员登录</p>
      </div>

      <label class="field">
        <span>管理员密码</span>
        <input v-model="password" type="password" autocomplete="current-password" autofocus>
      </label>

      <div v-if="!authConfigured" class="message">
        管理员密码尚未初始化，请先设置 WEB_FILE_BROWSER_ADMIN_PASSWORD 后重启服务。
      </div>
      <div v-if="errorMessage" class="message">{{ errorMessage }}</div>

      <button class="primary-button" type="submit" :disabled="loading || !password || !authConfigured">
        {{ loading ? "登录中" : "登录" }}
      </button>
    </form>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";
.login-page {
  @apply min-h-screen flex items-center justify-center px-4;
  background: var(--app-bg);
}

.login-panel {
  @apply w-full max-w-sm border rounded-lg shadow-sm p-6 flex flex-col gap-5;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
}

h1 {
  @apply text-2xl font-semibold;
  color: var(--app-text);
}

p {
  @apply mt-1 text-sm;
  color: var(--app-text-subtle);
}

.field {
  @apply flex flex-col gap-2 text-sm;
  color: var(--app-text-muted);
}

.field input {
  @apply h-10 rounded-md border px-3 outline-none;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text);
}

.field input:focus {
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.2));
}

.message {
  @apply rounded-md border px-3 py-2 text-sm text-red-700;
  border-color: color-mix(in srgb, #ef4444 36%, var(--app-border-soft));
  background: var(--app-danger-soft);
}

.primary-button {
  @apply h-10 rounded-md text-white font-medium disabled:cursor-not-allowed;
  background: var(--app-accent, #2563eb);
}

.primary-button:disabled {
  background: var(--app-control);
  color: var(--app-text-disabled);
}

.primary-button:hover:not(:disabled) {
  background: color-mix(in srgb, var(--app-accent, #2563eb) 88%, black);
}
</style>
