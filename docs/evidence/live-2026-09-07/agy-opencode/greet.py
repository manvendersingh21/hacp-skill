def greet(name: str) -> str:
    if name is None:
        raise ValueError("Name cannot be empty or None")
    stripped = name.strip()
    if not stripped:
        raise ValueError("Name cannot be empty or whitespace-only")
    return f"Hello, {stripped}!"
