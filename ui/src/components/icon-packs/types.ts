import type {Component} from "vue";

export interface AppIconComponentDefinition {
  kind: "component";
  component: Component;
  props?: Record<string, unknown>;
  className?: string;
  transform?: string;
}

export interface AppIconSymbolDefinition {
  kind: "symbol";
  symbol: string;
  className?: string;
  transform?: string;
}

export type AppIconDefinition = AppIconComponentDefinition | AppIconSymbolDefinition;

export interface AppIconPack {
  resolve(icon: string): AppIconDefinition | undefined;
}
