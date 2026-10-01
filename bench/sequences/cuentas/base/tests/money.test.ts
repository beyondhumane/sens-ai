import { expect, it } from "vitest";
import { formatMoney } from "../src/money";

it("writes euros the Spanish way", () => {
  expect(formatMoney(1250).replace(/ /g, " ")).toBe("12,50 €");
  expect(formatMoney(1234550).replace(/ /g, " ")).toBe("12.345,50 €");
});
