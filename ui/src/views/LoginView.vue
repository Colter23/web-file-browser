<script setup lang="ts">
import {computed, nextTick, onMounted, onUnmounted, ref, watch} from "vue";
import {useRoute, useRouter} from "vue-router";
import {getSession, login, setupPassword} from "../network/api";
import {isApiError} from "../network";
import {useI18n} from "../i18n";
import type {LocaleCode} from "../i18n";
import Icon from "../components/Icon.vue";

const route = useRoute();
const router = useRouter();
const {locale, localeOptions, setLocale, t} = useI18n();
const password = ref("");
const confirmPassword = ref("");
const loading = ref(false);
const checkingSession = ref(true);
const errorMessage = ref("");
const authConfigured = ref(true);
const retryAfterSeconds = ref(0);
const passwordInput = ref<HTMLInputElement | null>(null);
let retryTimer: number | undefined;
const setupMode = computed(() => !authConfigured.value);
const titleText = computed(() => setupMode.value ? t("login.initialSetup") : t("login.adminLogin"));
const passwordAutocomplete = computed(() => setupMode.value ? "new-password" : "current-password");
const isCoolingDown = computed(() => retryAfterSeconds.value > 0);
const submitText = computed(() => {
  if (checkingSession.value) return t("login.checkingSession");
  if (isCoolingDown.value) return t("login.retryAfter", {seconds: retryAfterSeconds.value});
  if (loading.value) return setupMode.value ? t("login.settingUp") : t("login.loggingIn");
  return setupMode.value ? t("login.setupAndEnter") : t("login.submit");
});
const canSubmit = computed(() => {
  if (checkingSession.value || loading.value || isCoolingDown.value || password.value.length < 8) return false;
  return !setupMode.value || confirmPassword.value.length >= 8;
});

const redirectPath = () => {
  const redirect = route.query.redirect;
  return typeof redirect === "string" && redirect.startsWith("/") && !redirect.startsWith("//") ? redirect : "/";
}

const selectLocale = (value: LocaleCode) => {
  setLocale(value);
}

const focusPasswordInput = async () => {
  await nextTick();
  passwordInput.value?.focus({preventScroll: true});
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

watch(setupMode, () => {
  confirmPassword.value = "";
  void focusPasswordInput();
});

watch([password, confirmPassword], () => {
  if (!loading.value && errorMessage.value) errorMessage.value = "";
});

onUnmounted(() => {
  if (retryTimer !== undefined) window.clearInterval(retryTimer);
})

const startLoginCooldown = (seconds: unknown) => {
  const parsed = Number(seconds);
  const initialSeconds = Number.isFinite(parsed) ? Math.max(1, Math.ceil(parsed)) : 1;
  if (retryTimer !== undefined) window.clearInterval(retryTimer);
  retryAfterSeconds.value = initialSeconds;
  retryTimer = window.setInterval(() => {
    retryAfterSeconds.value = Math.max(0, retryAfterSeconds.value - 1);
    if (retryAfterSeconds.value > 0 || retryTimer === undefined) return;
    window.clearInterval(retryTimer);
    retryTimer = undefined;
  }, 1000);
}

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
  authConfigured.value = wasSetupMode ? true : false;
  errorMessage.value = wasSetupMode
      ? t("login.alreadySetup")
      : t("login.needsSetup");
  return true;
}

const submit = async () => {
  if (loading.value || checkingSession.value || isCoolingDown.value || !validate()) return;
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
      if (isApiError(error) && error.reason === "LOGIN_FAILURE_COOLDOWN") {
        startLoginCooldown(error.params?.retryAfterSeconds);
      }
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
    <div class="login-language" role="group" :aria-label="t('app.language')">
      <icon icon="action.language" />
      <button
          v-for="option in localeOptions"
          :key="option.value"
          class="locale-option"
          :class="{active: locale === option.value}"
          type="button"
          :aria-pressed="locale === option.value"
          @click="selectLocale(option.value)">
        {{ t(option.labelKey) }}
      </button>
    </div>

    <form class="login-panel" :aria-busy="checkingSession || loading" @submit.prevent="submit">
      <header class="login-header">
        <span class="brand-mark">
          <icon icon="icon-password" size="1.35rem" />
        </span>
        <span class="brand-copy">
          <h1>Web File Browser</h1>
          <p>{{ titleText }}</p>
        </span>
      </header>

      <label class="field">
        <span>{{ t("login.adminPassword") }}</span>
        <input
            ref="passwordInput"
            v-model="password"
            type="password"
            minlength="8"
            required
            :autocomplete="passwordAutocomplete"
            :disabled="loading"
            :aria-invalid="Boolean(errorMessage)"
            :aria-describedby="errorMessage ? 'login-message' : undefined"
            autofocus>
      </label>

      <Transition name="field-slide">
        <label v-if="setupMode" class="field">
          <span>{{ t("login.confirmPassword") }}</span>
          <input
              v-model="confirmPassword"
              type="password"
              minlength="8"
              required
              autocomplete="new-password"
              :disabled="loading"
              :aria-invalid="Boolean(errorMessage)"
              :aria-describedby="errorMessage ? 'login-message' : 'setup-hint'">
        </label>
      </Transition>

      <Transition name="notice-slide">
        <div v-if="setupMode" id="setup-hint" class="hint">
          <icon icon="icon-password" />
          <span>{{ t("login.setupHint") }}</span>
        </div>
      </Transition>
      <Transition name="notice-slide">
        <div v-if="errorMessage" id="login-message" class="message" role="alert">
          <icon icon="action.warning" />
          <span>{{ errorMessage }}</span>
        </div>
      </Transition>

      <button class="primary-button" type="submit" :disabled="!canSubmit">
        <span v-if="checkingSession || loading" class="loading-dot"></span>
        {{ submitText }}
      </button>
    </form>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";
.login-page {
  @apply relative grid min-h-screen place-items-center px-4 py-12;
  background: var(--app-bg);
  color: var(--app-text);
}

.login-language {
  @apply fixed right-4 top-4 z-10 grid grid-cols-[1rem_auto_auto] items-center gap-1 rounded-md border p-1 text-xs shadow-sm backdrop-blur;
  border-color: color-mix(in srgb, var(--app-border) 50%, transparent);
  background: color-mix(in srgb, var(--app-panel-solid) 88%, transparent);
  color: var(--app-text-muted);
}

.locale-option {
  @apply inline-flex h-7 min-w-16 items-center justify-center rounded px-2 font-medium;
  color: var(--app-text-muted);
  transition: background 0.14s ease, color 0.14s ease, box-shadow 0.14s ease;
}

.locale-option:hover {
  background: var(--app-control-hover);
  color: var(--app-text);
}

.locale-option.active {
  @apply font-semibold;
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast, #ffffff);
  box-shadow: 0 1px 2px color-mix(in srgb, var(--app-accent, #2563eb) 20%, transparent);
}

.locale-option:focus-visible {
  @apply outline-none;
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.login-panel {
  @apply flex w-full max-w-[26rem] flex-col gap-5 rounded-xl border p-6 shadow-sm;
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-solid) 96%, transparent);
  box-shadow: var(--app-menu-shadow);
}

.login-header {
  @apply grid grid-cols-[3rem_minmax(0,1fr)] items-center gap-3;
}

.brand-mark {
  @apply grid h-12 w-12 place-items-center rounded-xl border;
  border-color: var(--app-accent-border);
  background: var(--app-accent-soft);
  color: var(--app-accent, #2563eb);
}

.brand-copy {
  @apply min-w-0;
}

h1 {
  @apply truncate text-2xl font-semibold leading-tight;
  color: var(--app-text);
}

p {
  @apply mt-1 text-sm;
  color: var(--app-text-subtle);
}

.field {
  @apply flex flex-col gap-2 text-sm font-medium;
  color: var(--app-text-muted);
}

.field input {
  @apply h-11 rounded-md border px-3 text-sm outline-none;
  border-color: color-mix(in srgb, var(--app-border) 64%, transparent);
  background: var(--app-control-solid);
  color: var(--app-text);
  transition: border-color 0.14s ease, background 0.14s ease, box-shadow 0.14s ease;
}

.field input:hover:not(:disabled) {
  border-color: color-mix(in srgb, var(--app-accent, #2563eb) 28%, var(--app-border));
}

.field input:focus {
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.2));
}

.field input:disabled {
  cursor: wait;
  color: var(--app-text-disabled);
  background: var(--app-control);
}

.field input[aria-invalid="true"] {
  border-color: var(--app-danger-border);
}

.message {
  @apply grid grid-cols-[1rem_minmax(0,1fr)] items-start gap-2 rounded-md border px-3 py-2 text-sm;
  border-color: var(--app-danger-border);
  background: var(--app-danger-soft);
  color: var(--app-danger-text);
}

.hint {
  @apply grid grid-cols-[1rem_minmax(0,1fr)] items-start gap-2 rounded-md border px-3 py-2 text-sm;
  border-color: var(--app-accent-border);
  background: color-mix(in srgb, var(--app-accent-soft, #eff6ff) 72%, var(--app-panel-solid));
  color: color-mix(in srgb, var(--app-accent, #2563eb) 84%, var(--app-text));
}

.primary-button {
  @apply inline-flex h-11 items-center justify-center gap-2 rounded-md font-semibold disabled:cursor-not-allowed;
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast);
  transition: background 0.14s ease, color 0.14s ease, transform 0.12s ease, box-shadow 0.14s ease;
}

.primary-button:disabled {
  background: var(--app-control);
  color: var(--app-text-disabled);
}

.primary-button:hover:not(:disabled) {
  background: var(--app-accent-strong);
  box-shadow: 0 8px 20px color-mix(in srgb, var(--app-accent, #2563eb) 18%, transparent);
}

.primary-button:active:not(:disabled) {
  transform: translateY(1px);
}

.primary-button:focus-visible {
  @apply outline-none;
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.loading-dot {
  @apply h-2 w-2 rounded-full;
  background: currentColor;
  animation: login-loading-pulse 0.8s ease-in-out infinite alternate;
}

.field-slide-enter-active,
.field-slide-leave-active,
.notice-slide-enter-active,
.notice-slide-leave-active {
  transition: opacity 0.14s ease, transform 0.16s ease;
}

.field-slide-enter-from,
.field-slide-leave-to,
.notice-slide-enter-from,
.notice-slide-leave-to {
  opacity: 0;
  transform: translateY(-0.25rem);
}

@keyframes login-loading-pulse {
  from {
    opacity: 0.45;
    transform: scale(0.86);
  }

  to {
    opacity: 1;
    transform: scale(1);
  }
}

@media (max-width: 520px) {
  .login-page {
    @apply items-start pt-20;
  }

  .login-language {
    @apply right-3 top-3;
  }

  .login-panel {
    @apply p-5;
  }
}

@media (prefers-reduced-motion: reduce) {
  .field-slide-enter-active,
  .field-slide-leave-active,
  .notice-slide-enter-active,
  .notice-slide-leave-active,
  .primary-button,
  .field input,
  .locale-option {
    transition: none;
  }

  .loading-dot {
    animation: none;
  }
}
</style>
