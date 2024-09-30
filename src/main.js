const { invoke } = window.__TAURI__.tauri;

let monacoEditor;

function initializeMonacoEditor() {
  return new Promise((resolve) => {
    require.config({ paths: { vs: "monaco-editor/min/vs" } });
    require(["vs/editor/editor.main"], function () {
      monacoEditor = monaco.editor.create(
        document.getElementById("monaco-editor-container"),
        {
          value: ["select * from 'data/etfs.csv'"].join("\n"),
          language: "sql",
          theme: "vs-light",
          automaticLayout: true,
          scrollBeyondLastLine: false,
          minimap: { enabled: false },
          fontSize: 14,
          lineNumbers: "on",
          wordWrap: "off",
          scrollbar: {
            vertical: "hidden",
            horizontal: "hidden",
            verticalScrollbarSize: 0,
            horizontalScrollbarSize: 0,
          },
          padding: {
            top: 10,
          },
        },
      );
      resolve();
    });
  });
}

async function create_table() {
  if (!monacoEditor) {
    console.error("Monaco Editor not initialized");
    return;
  }
  const query = monacoEditor.getValue();
  const tableContainer = document.querySelector("#table-container");
  try {
    const htmlContent = await invoke("create_table", { query });
    tableContainer.innerHTML = htmlContent;
  } catch (error) {
    showErrorModal(error);
  }
}

function showErrorModal(errorMessage) {
  // Escape the error message to prevent XSS
  const escapedErrorMessage = escapeHtml(errorMessage);

  const modal = document.createElement("div");
  modal.innerHTML = `
    <div class="fixed z-10 inset-0 overflow-y-auto" aria-labelledby="modal-title" role="dialog" aria-modal="true">
      <div class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
        <div class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" aria-hidden="true"></div>
        <span class="hidden sm:inline-block sm:align-middle sm:h-screen" aria-hidden="true">&#8203;</span>
        <div class="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
          <div class="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
            <div class="sm:flex sm:items-start">
              <div class="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left">
                <h3 class="text-lg leading-6 font-medium text-gray-900" id="modal-title">
                  Error
                </h3>
                <div class="mt-2">
                  <p class="text-sm text-gray-500">
                    ${escapedErrorMessage}
                  </p>
                </div>
              </div>
            </div>
          </div>
          <div class="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
            <button type="button" class="mt-3 w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-blue-600 text-base font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm" onclick="this.closest('.fixed').remove()">
              Close
            </button>
          </div>
        </div>
      </div>
    </div>
  `;
  document.body.appendChild(modal);
}

function escapeHtml(unsafe) {
  return unsafe
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

// document.addEventListener("DOMContentLoaded", () => {
//   const textarea = document.querySelector("#input-box");

//   // Prevent smart quote conversion
//   textarea.addEventListener("input", function (e) {
//     const start = this.selectionStart;
//     const end = this.selectionEnd;
//     const value = this.value;

//     this.value = value.replace(/['']/g, "'").replace(/[""]/g, '"');

//     // Restore the cursor position
//     this.setSelectionRange(start, end);
//   });
//
async function updateFileExplorer() {
  try {
    const htmlContent = await invoke("list_files");
    document.querySelector("#file-explorer").innerHTML = htmlContent;

    // Add click event listeners to SQL files
    document.querySelectorAll(".sql-file").forEach((element) => {
      element.addEventListener("click", async (e) => {
        e.preventDefault();
        const filePath = e.currentTarget.dataset.path;
        const fileContent = await invoke("read_file", { path: filePath });
        monacoEditor.setValue(fileContent);
      });
    });
  } catch (error) {
    console.error("Error updating file explorer:", error);
  }
}

async function init() {
  await initializeMonacoEditor();
  await updateFileExplorer();

  document
    .querySelector("#create-table-form")
    .addEventListener("submit", (e) => {
      e.preventDefault();
      create_table();
    });
}

init();
