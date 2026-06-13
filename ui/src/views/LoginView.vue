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
  @apply min-h-screen flex items-center justify-center bg-slate-100 px-4
}

.login-panel {
  @apply w-full max-w-sm bg-white border border-slate-200 rounded-lg shadow-sm p-6 flex flex-col gap-5
}

h1 {
  @apply text-2xl font-semibold text-slate-900
}

p {
  @apply mt-1 text-sm text-slate-500
}

.field {
  @apply flex flex-col gap-2 text-sm text-slate-700
}

.field input {
  @apply h-10 rounded-md border border-slate-300 px-3 outline-none focus:border-blue-500
}

.message {
  @apply rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700
}

.primary-button {
  @apply h-10 rounded-md bg-blue-600 text-white font-medium hover:bg-blue-700 disabled:bg-slate-300 disabled:cursor-not-allowed
}
</style>
