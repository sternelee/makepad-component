# A2UI Raycast Tooling Baseline

This document defines the Raycast-style tool set used by the embedded A2UI bridge in `raycast-launcher`.

Reference UI model: Raycast API UI components (`List`, `Detail`, `Form`, `ActionPanel`, `Grid`, form items).  
Source: https://developers.raycast.com/api-reference/user-interface

## Container Tools
- `create_list`: list container with `children`, optional `direction`.
- `create_detail`: markdown + metadata detail page, optional `actionsId`.
- `create_form`: form container with `children`, optional `actionsId`, `navigationTitle`.
- `create_action_panel`: action group container.
- `create_grid`: grid container with `children`, optional `columns`.
- `create_column` / `create_row` / `create_card`: generic layout primitives.

## Item and Action Tools
- `create_list_item`: list/grid item wrapper (`childId`, optional icon/accessory/action).
- `create_button`: action entry (used standalone or inside `ActionPanel`).
- `create_text`: reusable label/title/description node.

## Form Field Tools
- `create_textfield`
- `create_password_field`
- `create_text_area`
- `create_date_picker`
- `create_dropdown`
- `create_dropdown_item`
- `create_dropdown_section`
- `create_tag_picker`
- `create_tag_picker_item`
- `create_file_picker`
- `create_checkbox`
- `create_slider`

## Data and Render Tools
- `set_data`: initialize path-bound data model values.
- `render_ui`: finalize output with `rootId` (must be called last).

## Template Tools
- `create_template_command_list`: one-call scaffold for command launcher UX (`title + optional search + grouped list + actionPanel`).
- `create_template_form_submit`: one-call scaffold for submit workflows (`form + submit/cancel actions`).
- `create_template_list_detail`: one-call scaffold for plugin pattern (`header + list + detail + shared actions`).

## Notes
- Tool calls are converted by `A2uiBuilder` into A2UI messages:
  - `beginRendering`
  - `surfaceUpdate`
  - `dataModelUpdate`
- This gives a stable foundation for richer Raycast-like plugin generation without requiring raw A2UI JSON output from the model.
