# Rust Text Editor (Vim-inspired experiment)

## Overview
This is a small text editor written in Rust, inspired by my interest in how **Vim** works.  
While much of the low-level code was generated with AI assistance, I directed the design and feature set.  
Instead of asking for a complete editor, I iteratively requested **specific functions** and integrated them into the project.

## Features
- Buffer system for storing and rendering text
- Terminal output clears and re-renders each frame (text is the only thing visible on screen)
- Basic text navigation and editing commands

## Learning Focus
- How buffers are used to represent and render text
- How terminal-based editors refresh the screen without artifacts
- How state management in Rust (ownership, borrowing) plays into real systems

## Reflection
This project wasn’t about building a fully original editor, but about **learning by directing**.  
By steering AI-generated functions toward a working program, I gained insight into:
- The architecture behind text editors
- Rust’s approach to memory and safety in a real application
- The challenge of stitching multiple AI-suggested features into a coherent system
