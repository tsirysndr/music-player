import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useForm } from "react-hook-form";
import { describe, expect, it, vi } from "vitest";
import Select from "./Select";
import TextField, { TextAreaField } from "./TextField";

describe("TextField", () => {
  it("associates its label with the input", () => {
    render(<TextField label="NAME" />);
    expect(screen.getByLabelText("NAME")).toBeInTheDocument();
  });

  it("shows a hint under the field", () => {
    render(<TextField label="LIMIT" hint="0 means every track" />);
    expect(screen.getByText("0 means every track")).toBeInTheDocument();
  });

  /** An error replaces the hint — showing both is two competing instructions. */
  it("shows the error instead of the hint, and marks the field invalid", () => {
    render(
      <TextField label="LIMIT" hint="0 means every track" error="Must be a number" />
    );

    expect(screen.getByText("Must be a number")).toBeInTheDocument();
    expect(screen.queryByText("0 means every track")).toBeNull();
    expect(screen.getByLabelText("LIMIT")).toHaveAttribute(
      "aria-invalid",
      "true"
    );
  });

  it("points at its error message for assistive tech", () => {
    render(<TextField label="NAME" error="Give your playlist a name" />);
    const input = screen.getByLabelText("NAME");
    const describedBy = input.getAttribute("aria-describedby");

    expect(describedBy).toBeTruthy();
    expect(document.getElementById(describedBy!)).toHaveTextContent(
      "Give your playlist a name"
    );
  });

  /**
   * Every form in the app is a react-hook-form, and `register()` hands back a
   * ref the field has to pass through or the value is never read.
   */
  it("forwards its ref, so react-hook-form can register it", async () => {
    const user = userEvent.setup();
    const onValid = vi.fn();

    const Form = () => {
      const { register, handleSubmit } = useForm<{ name: string }>({
        defaultValues: { name: "" },
      });
      return (
        <form onSubmit={handleSubmit(onValid)}>
          <TextField label="NAME" {...register("name")} />
          <button type="submit">Save</button>
        </form>
      );
    };

    render(<Form />);
    await user.type(screen.getByLabelText("NAME"), "Late night");
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(onValid).toHaveBeenCalledWith(
      expect.objectContaining({ name: "Late night" }),
      expect.anything()
    );
  });
});

describe("TextAreaField", () => {
  it("associates its label and shows an error", () => {
    render(<TextAreaField label="DESCRIPTION" error="Too long" />);
    expect(screen.getByLabelText("DESCRIPTION")).toHaveAttribute(
      "aria-invalid",
      "true"
    );
    expect(screen.getByText("Too long")).toBeInTheDocument();
  });

  it("forwards its ref", async () => {
    const user = userEvent.setup();
    const onValid = vi.fn();

    const Form = () => {
      const { register, handleSubmit } = useForm<{ description: string }>({
        defaultValues: { description: "" },
      });
      return (
        <form onSubmit={handleSubmit(onValid)}>
          <TextAreaField label="DESCRIPTION" {...register("description")} />
          <button type="submit">Save</button>
        </form>
      );
    };

    render(<Form />);
    await user.type(screen.getByLabelText("DESCRIPTION"), "For the small hours");
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(onValid).toHaveBeenCalledWith(
      expect.objectContaining({ description: "For the small hours" }),
      expect.anything()
    );
  });
});

describe("Select", () => {
  const OPTIONS = [
    { value: "", label: "Library order" },
    { value: "random", label: "Random" },
    { value: "year", label: "Year" },
  ];

  it("associates its label and lists its options", () => {
    render(<Select label="SORT BY" options={OPTIONS} />);
    const select = screen.getByLabelText("SORT BY");

    expect(select).toBeInTheDocument();
    expect(screen.getAllByRole("option")).toHaveLength(3);
  });

  it("selects an option", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(
      <Select label="SORT BY" options={OPTIONS} defaultValue="" onChange={onChange} />
    );

    await user.selectOptions(screen.getByLabelText("SORT BY"), "random");
    expect(screen.getByLabelText("SORT BY")).toHaveValue("random");
    expect(onChange).toHaveBeenCalled();
  });

  it("marks itself invalid and shows the message", () => {
    render(<Select label="SORT BY" options={OPTIONS} error="Pick one" />);
    expect(screen.getByLabelText("SORT BY")).toHaveAttribute(
      "aria-invalid",
      "true"
    );
    expect(screen.getByText("Pick one")).toBeInTheDocument();
  });
});
