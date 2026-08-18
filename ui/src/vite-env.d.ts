/// <reference types="vite/client" />

// Without this reference, a side-effect import of a stylesheet
// (`import "./index.css"`) has no type declaration, and TypeScript rejects it
// outright from 6.0 on with TS2882. The reference also brings in the types for
// `import.meta.env`.
