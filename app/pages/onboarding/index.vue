<script lang="ts" setup>
import { Button } from "~/components/ui/button";
import { Alert, AlertDescription } from "~/components/ui/alert/index";
import LogoType from "~/assets/images/logotype.vue";
import {
  Stepper,
  StepperIndicator,
  StepperItem,
  StepperTitle,
  StepperSeparator,
  StepperTrigger,
} from "~/components/ui/stepper";
import { RadioGroup } from "~/components/ui/radio-group";
import { RadioGroupIndicator, RadioGroupItem } from "reka-ui";
import { Badge } from "~/components/ui/badge";
import { InputField } from "~/components/ui/form";
import { invoke } from "@tauri-apps/api/core";
import { useLicense } from "~/composables/useLicense";

definePageMeta({
  layout: "empty",
});

const route = useRoute();
const router = useRouter();
const stepper = useTemplateRef("stepper");
const { activate, startTrial, isLoading } = useLicense();

const step = computed({
  get: () => (route.query.step ? Number(route.query.step) : 0),
  set: (value: number) => {
    router.replace({ query: { ...route.query, step: value.toString() } });
  },
});

const steps = [
  {
    title: "onboarding.license.title",
  },
  {
    title: "onboarding.customize.title",
  },
  {
    title: "onboarding.complete.title",
  },
];

// Form state
const licenseMode = ref("trial");
const email = ref("");
const licenseKey = ref("");
const resultMessage = ref("");
const resultType = ref<"success" | "error" | null>(null);

const licenseModes = [
  {
    value: "trial",
    badge: "onboarding.license.licenseModes.trial.badge",
    label: "onboarding.license.licenseModes.trial.title",
    description: "onboarding.license.licenseModes.trial.description",
    benefits: "onboarding.license.benefits",
  },
  {
    value: "activate",
    label: "onboarding.license.licenseModes.activate.title",
    description: "onboarding.license.licenseModes.activate.description",
    cta: "onboarding.license.licenseModes.activate.cta",
  },
];

// Validation
const isValidEmail = computed(() => {
  if (!email.value) return false;
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email.value);
});

const canSubmit = computed(() => {
  if (isLoading.value) return false;

  if (licenseMode.value === "trial") {
    return isValidEmail.value;
  } else {
    return isValidEmail.value && licenseKey.value.trim().length > 0;
  }
});

// Clear result message when switching modes
watch(licenseMode, () => {
  resultMessage.value = "";
  resultType.value = null;
  licenseKey.value = "";
});

// Handle form submission
const handleSubmit = async () => {
  if (!canSubmit.value) return;

  resultMessage.value = "";
  resultType.value = null;

  if (licenseMode.value === "trial") {
    const response = await startTrial(email.value);

    if (response.success) {
      resultType.value = "success";
      resultMessage.value = response.message;

      // Move to next step after successful trial
      setTimeout(() => {
        step.value = 1;
      }, 1500);
    } else {
      resultType.value = "error";
      resultMessage.value = response.message;
    }
  } else {
    const response = await activate(licenseKey.value.trim());

    if (response.success) {
      resultType.value = "success";
      resultMessage.value = response.message;

      // Move to next step after successful activation
      setTimeout(() => {
        step.value = 1;
      }, 1500);
    } else {
      resultType.value = "error";
      resultMessage.value = response.message;
    }
  }
};

const openPricing = () => {
  invoke("open_external_url", { url: "https://www.ravnmail.com/pricing" });
};
</script>

<template>
  <div
    class="flex items-center justify-center h-full overflow-auto p-4"
    style="
      background: url(&quot;/mike-yukhtenko-wfh8dDlNFOk-unsplash.jpg&quot;)
        center/cover no-repeat fixed;
    "
  >
    <LogoType class="text-white h-8 fixed left-12 top-12" />
    <Markdown
      class="fixed bottom-3 right-3 text-2xs opacity-50"
      content="Photo by [Adrien Olichon](https://unsplash.com/@adrienolichon) on [Unsplash](https://unsplash.com/photos/grey-sand-wave-RCAhiGJsUUE)"
    />
    <div
      class="relative z-10 bg-dialog-background w-xl border border-popover-border shadow-lg rounded-xl overflow-clip"
    >
      <Stepper
        ref="stepper"
        v-model="step"
        class="flex w-full items-start gap-2 bg-popover px-6 py-3"
        orientation="horizontal"
      >
        <StepperItem
          v-for="(item, index) in steps"
          :key="index"
          :step="index"
          class="relative w-full flex items-center justify-center"
        >
          <StepperTrigger class="flex items-center flex-row gap-2">
            <StepperIndicator>{{ index + 1 }}</StepperIndicator>
            <StepperTitle>{{ $t(item.title) }}</StepperTitle>
          </StepperTrigger>
          <StepperSeparator
            v-if="index < steps.length - 1"
            class="absolute left-[calc(75%+20px)] right-[calc(-25%+10px)] top-3 block h-px shrink-0 rounded-full bg-muted group-data-[state=completed]:bg-primary"
          />
        </StepperItem>
      </Stepper>
      <div class="p-6">
        <div v-if="step === 0" class="flex flex-col gap-6">
          <!-- Header -->
          <div>
            <h1 class="text-primary font-medium">
              {{ $t("onboarding.license.header") }}
            </h1>
            <p class="text-muted-foreground text-sm">
              {{ $t("onboarding.license.description") }}
            </p>
          </div>

          <!-- Result Message -->
          <Alert
            v-if="resultMessage"
            :class="[
              resultType === 'success'
                ? 'border-green-500 bg-green-50 dark:bg-green-900/20'
                : 'border-red-500 bg-red-50 dark:bg-red-900/20',
            ]"
          >
            <AlertDescription
              :class="
                resultType === 'success'
                  ? 'text-green-800 dark:text-green-200'
                  : 'text-red-800 dark:text-red-200'
              "
            >
              {{ resultMessage }}
            </AlertDescription>
          </Alert>

          <!-- License Mode Selection -->
          <RadioGroup
            v-model="licenseMode"
            class="grid grid-cols-1 gap-2 w-full"
          >
            <RadioGroupItem
              v-for="mode in licenseModes"
              :key="mode.value"
              :value="mode.value"
              :disabled="isLoading"
              class="flex gap-3 p-4 border border-border rounded-lg text-left transition-all hover:border-primary/50 data-[state=checked]:border-primary data-[state=checked]:bg-primary/5 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <div
                class="peer flex items-center justify-center size-5 rounded-full bg-input shadow focus:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 mt-0.5"
              >
                <RadioGroupIndicator class="flex items-center justify-center">
                  <span class="size-2 bg-primary rounded-full" />
                </RadioGroupIndicator>
              </div>
              <div class="flex flex-col w-full gap-2">
                <div class="flex items-start">
                  <h3 class="font-medium text-primary flex-1">
                    {{ $t(mode.label) }}
                  </h3>
                  <Badge
                    v-if="mode.badge"
                    class="ml-auto"
                    size="sm"
                    variant="primary"
                    >{{ $t(mode.badge) }}
                  </Badge>
                </div>
                <p class="text-sm text-muted-foreground">
                  {{ $t(mode.description) }}
                </p>
                <div
                  v-if="mode.value === 'trial' && mode.benefits"
                  class="mt-2 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-md p-3"
                >
                  <ul
                    class="space-y-1 text-sm text-blue-800 dark:text-blue-200"
                  >
                    <li v-for="benefit in mode.benefits.split('\n')" :key="benefit">
                      ✓ {{ $t(benefit) }}
                    </li>
                  </ul>
                </div>
                <div v-if="mode.cta" class="mt-1">
                  <Button
                    size="sm"
                    variant="outline"
                    @click.prevent.stop="openPricing"
                  >
                    {{ $t(mode.cta) }}
                  </Button>
                </div>
              </div>
            </RadioGroupItem>
          </RadioGroup>
          <div class="w-full flex flex-col gap-4">
            <InputField
              v-if="licenseMode === 'trial'"
              v-model="email"
              :label="$t('onboarding.license.emailLabel')"
              :placeholder="$t('onboarding.license.emailPlaceholder')"
              :description="$t('onboarding.license.emailHint')"
              :disabled="isLoading"
              name="email"
              type="email"
              @keyup.enter="handleSubmit"
            />
            <InputField
              v-else
              v-model="licenseKey"
              :label="$t('onboarding.license.licenseKeyLabel')"
              :placeholder="$t('onboarding.license.licenseKeyPlaceholder')"
              :description="$t('onboarding.license.licenseKeyHint')"
              :disabled="isLoading"
              name="licenseKey"
              type="text"
              @keyup.enter="handleSubmit"
            />
          </div>
          <Button
            :disabled="!canSubmit"
            class="w-full"
            size="lg"
            variant="primary"
            @click="handleSubmit"
          >
            <span v-if="isLoading" class="flex items-center gap-2">
              <svg
                class="animate-spin h-5 w-5"
                fill="none"
                viewBox="0 0 24 24"
                xmlns="http://www.w3.org/2000/svg"
              >
                <circle
                  class="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  stroke-width="4"
                />
                <path
                  class="opacity-75"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  fill="currentColor"
                />
              </svg>
              {{ $t("onboarding.license.processing") }}
            </span>
            <span v-else>
              {{
                licenseMode === "trial"
                  ? $t("onboarding.license.startTrial")
                  : $t("onboarding.license.activateLicense")
              }}
            </span>
          </Button>

          <Markdown
            :content="$t('onboarding.license.terms')"
            class="text-center text-xs text-muted-foreground"
          />
        </div>

        <div v-else-if="step === 1" class="flex flex-col gap-6">
          <div>
            <h1 class="text-primary font-medium">
              {{ $t("onboarding.customize.header") }}
            </h1>
            <p class="text-muted-foreground text-sm">
              {{ $t("onboarding.customize.description") }}
            </p>
          </div>
        </div>

        <div v-else-if="step === 2" class="flex flex-col gap-6">
          <div>
            <h1 class="text-primary font-medium">
              {{ $t("onboarding.complete.header") }}
            </h1>
            <p class="text-muted-foreground text-sm">
              {{ $t("onboarding.complete.description") }}
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
