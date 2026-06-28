<script setup lang="ts">
import {computed, onMounted, ref} from "vue";
import {useRoute, useRouter} from "vue-router";
import {getSession, login, setupPassword} from "../network/api";
import {isApiError} from "../network";
import {useI18n} from "../i18n";

const route = useRoute();
const router = useRouter();
const {t} = useI18n();
const password = ref("");
const confirmPassword = ref("");
const loading = ref(false);
const checkingSession = ref(true);
const errorMessage = ref("");
const authConfigured = ref(true);
const setupMode = computed(() => !authConfigured.value);
const titleText = computed(() => setupMode.value ? t("login.initialSetup") : t("login.adminLogin"));
const passwordAutocomplete = computed(() => setupMode.value ? "new-password" : "current-password");
const submitText = computed(() => {
  if (loading.value) return setupMode.value ? t("login.settingUp") : t("login.loggingIn");
  return setupMode.value ? t("login.setupAndEnter") : t("login.submit");
});
const canSubmit = computed(() => {
  if (checkingSession.value || loading.value || password.value.length < 8) return false;
  return !setupMode.value || confirmPassword.value.length >= 8;
});

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
    errorMessage.value = error instanceof Error ? error.message : t("login.sessionFailed");
  } finally {
    checkingSession.value = false;
  }
})

const validate = () => {
  if (password.value.length < 8) {
    errorMessage.value = t("login.passwordTooShort");
    return false;
  }
  if (setupMode.value && password.value !== confirmPassword.value) {
    errorMessage.value = t("login.passwordMismatch");
    return false;
  }
  return true;
}

const handleAuthConflict = (error: unknown, wasSetupMode: boolean) => {
  if (!isApiError(error) || error.status !== 409) return false;
  confirmPassword.value = "";
  authConfigured.value = wasSetupMode;
  errorMessage.value = wasSetupMode
      ? t("login.alreadySetup")
      : t("login.needsSetup");
  return true;
}

const submit = async () => {
  if (loading.value || checkingSession.value || !validate()) return;
  loading.value = true;
  errorMessage.value = "";
  const wasSetupMode = setupMode.value;
  try {
    const session = wasSetupMode
        ? await setupPassword(password.value)
        : await login(password.value);
    authConfigured.value = session.authConfigured;
    await router.replace(redirectPath());
  } catch (error) {
    if (!handleAuthConflict(error, wasSetupMode)) {
      errorMessage.value = error instanceof Error
          ? error.message
          : wasSetupMode ? t("login.setupFailed") : t("login.loginFailed");
    }
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
        <p>{{ titleText }}</p>
      </div>

      <label class="field">
        <span>{{ t("login.adminPassword") }}</span>
        <input
            v-model="password"
            type="password"
            minlength="8"
            :autocomplete="passwordAutocomplete"
            autofocus>
      </label>

      <label v-if="setupMode" class="field">
        <span>{{ t("login.confirmPassword") }}</span>
        <input v-model="confirmPassword" type="password" minlength="8" autocomplete="new-password">
      </label>

      <div v-if="setupMode" class="hint">
        {{ t("login.setupHint") }}
      </div>
      <div v-if="errorMessage" class="message">{{ errorMessage }}</div>

      <button class="primary-button" type="submit" :disabled="!canSubmit">
        {{ submitText }}
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
  @apply rounded-md border px-3 py-2 text-sm;
  border-color: var(--app-danger-border);
  background: var(--app-danger-soft);
  color: var(--app-danger-text);
}

.hint {
  @apply rounded-md border px-3 py-2 text-sm;
  border-color: var(--app-accent-border);
  background: var(--app-accent-soft);
  color: var(--app-accent);
}

.primary-button {
  @apply h-10 rounded-md font-medium disabled:cursor-not-allowed;
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast);
}

.primary-button:disabled {
  background: var(--app-control);
  color: var(--app-text-disabled);
}

.primary-button:hover:not(:disabled) {
  background: var(--app-accent-strong);
}
</style>
