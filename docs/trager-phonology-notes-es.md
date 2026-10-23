# Notas sobre la fonología del náhuatl de la Huasteca cerca de Chicontepec

## Ortografías usadas para escribir la lengua y sus implicaciones en la app Yoltok

**Autor:** Ed Trager  
**Fecha:** 2021-12-21  
**Última actualización:** 2022-02-22

---

## 1. Conceptos y definiciones

Definamos algunos términos lingüísticos y características del náhuatl:

- Una **palabra** se compone de una o más sílabas. Por ejemplo, la palabra `/t͡ɬah.tol.li/` tiene tres sílabas.
- Un **fonema** es una representación abstracta de un sonido en una palabra.
- Un **grafema** es una representación de un sonido en la escritura de una lengua.
- Un **morfema** es la unidad más pequeña de una lengua que tiene significado léxico o gramatical y que no puede dividirse más.

### Ejemplo en inglés

- *heal* `[ˈhiːl]`
- *health* `[ˈhɛlθ]`
- *healthy* `[ˈhɛlθi]`

Un morfema puede tener pronunciaciones diferentes en el habla, por ejemplo:

```text
[ˈhiːl]
[ˈhɛl-]
```

Hay fonemas consonánticos y fonemas vocálicos.

### Estructura de la sílaba

Una sílaba puede componerse de tres partes:

- **Inicio (onset):** consonante (C)
- **Núcleo:** vocal (V)
- **Coda:** consonante (C)

| Sílaba | Inicio | Núcleo | Coda |
|---|---|---|---|
| `t͡ɬen` | `t͡ɬ` | `e` | `n` |
| `aːt͡ɬ` | ∅ | `aː` | `t͡ɬ` |
| `mits` | `m` | `i` | `ts` |
| `ne` | `n` | `e` | ∅ |
| `aː` | ∅ | `aː` | ∅ |

### Restricciones fonológicas

- El náhuatl sólo admite consonantes simples.
- No permite agrupaciones consonánticas nativas.
- Los siguientes dígrafos representan consonantes únicas:

| Dígrafo | Representación |
|---|---|
| `tl` | /t͡ɬ/ |
| `tz` | /t͡s/ |
| `ts` | /t͡s/ |
| `ch` | /t͡ʃ/ |
| `hu` | /w/ |
| `uh` | /w/ (en coda) |
| `cu` | /kʷ/ |
| `uc` | /kʷ/ (en coda) |

El náhuatl de la Huasteca no tiene diptongos fonológicos. La letra `u` no representa una vocal independiente; forma parte de los dígrafos `hu`, `uh`, `cu` y `uc`.

### Forma subyacente y forma superficial

Cada palabra posee:

- **Forma subyacente:** representación mental del hablante, compuesta por una serie de fonemas.
- **Forma superficial:** realización fonética producida, modificada por reglas alofónicas.

**Convenciones:**

```text
/tonatiw/   ← forma subyacente
[tonatih]   ← forma superficial
```

Las formas subyacentes se escriben entre barras: `/ /`.  
Las formas superficiales se escriben entre corchetes: `[ ]`.

...

## 2. Tabla de correspondencias entre ortografías

La siguiente sección describe las correspondencias entre:

- ACK
- Trager
- Hasler
- SEP
- AFI (IPA)
- Notas

ACK sigue convenciones españolas:

```text
ca  = /ka/
que = /ke/
qui = /ki/
co  = /ko/
```

`cu` aparece en posición de inicio silábico y `uc` en posición de coda.

ACK también sigue estas convenciones:

```text
za = /sa/
ce = /se/
ci = /si/
zo = /so/
```

`hu` aparece en posición de inicio silábico y `uh` en posición de coda.

El punto inferior en el alfasilabario de Trager representa /h/ en coda silábica.

## 3. Herramientas y algoritmos

### 3.1. Herramienta de conversión ortográfica NOCE

NOCE (*Nahuatl Orthography Conversion Engine*) distingue dos categorías:

- **Ortografías morfofonémicas (M):**
  - Andrews–Campbell–Karttunen (ACK)
  - Trager
- **Ortografías fonéticas (F):**
  - Hasler
  - SEP

La conversión de **M → F** puede realizarse con alta precisión porque las reglas alofónicas son conocidas.

La conversión de **F → M** es inherentemente ambigua y sólo es posible parcialmente.

...

## 4. Alófonos en náhuatl de la Huasteca

### 4.1. Alófonos de /w/

Reglas:

```text
/w/ → [w] al inicio de sílaba
/w/ → [h] al final de sílaba
```

**Ejemplo:**

| Clase | Inicio de sílaba | Final de sílaba |
|---|---|---|
| AFI | `[kiˈko·wa]` | `[kiˈkoh·ki]` |
| ACK | `quicohua` | `quicouhqui` |
| Hasler | `kikowa` | `kikohki` |
| SEP | `kikoua` | `kikojki` |

**Conversión por máquina:**

- ✅ M → F: `/w/ → [h]`
- ☯ F → M: `[h] → /w/` sólo en algunos casos

### 4.2. Alófonos de /kʷ/

```text
/kʷ/ → [kʷ] al inicio de sílaba
/kʷ/ → [k]  al final de sílaba
/kʷ/ → [h]  antes de /k/
```

**Conversión por máquina:**

- ✅ M → F: `/kʷ/ → [k]`
- ✅ M → F: `/kʷ/ → [h]` antes de `/k/`
- ❌ F → M: `[k] → /kʷ/`

### 4.3. /n/ al final de palabra

En muchos contextos finales:

```text
/n/ → [h]
```

**Ejemplo:**

| Clase | Forma general | Forma posesiva |
|---|---|---|
| ACK | `cintli` | `nosin` |
| Hasler | `sintli` | `nosih` |
| SEP | `sintli` | `nosij` |

**Conversión por máquina:**

- ✅ M → F: `/n/ → [h]`
- ✅ M → F: distingue `tlen`, `tlan`, `axtlen`
- ❌ F → M: `[h] → /n/`

### 4.4. Lenición de /k/ en posición de coda

```text
/k/ → [h]
```

cuando precede una sílaba que comienza con consonante.

**Ejemplo:**

| Contexto | ACK | SEP |
|---|---|---|
| Antes de vocal | `niquihtoa` | `nikijtoa` |
| Antes de consonante | `nicnequi` | `nijneki` |

**Conversión por máquina:**

- ✅ M → F: `/k/ → [h]`
- ☯ F → M: `[h] → /k/` en contextos limitados

## 5. Recursos consultados

- Aguilar, Andrés Ehecatl. *A Phonological Description of Huasteca Nahuatl from Chicontepec, Veracruz*. California State University, Northridge (2013).
- Sánchez et al. *Tlahtolnechikolli Diccionario Nawatl Moderno – Español de la Sierra de Zongolica* (2004).
- Sullivan et al. *Tlahtolxitlauhcayotl* (2016).
- Toledo et al. *Conozcamos Nuestra Lengua Náhuatl – Ma Tikixmatikaj Tonauatlajtol* (1996).
